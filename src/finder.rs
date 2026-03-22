use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::{fs, io};

/// Find files recursively in directories (and files), excluding certain paths.
/// Absolute paths are returned.
pub fn find_files(
    include: &[impl AsRef<Path>],
    exclude: &[impl AsRef<Path>],
) -> Result<BTreeSet<PathBuf>, io::Error> {
    let include_paths_set = include
        .into_iter()
        .map(|p| p.as_ref().canonicalize())
        .collect::<io::Result<BTreeSet<_>>>()?;
    let exclude_paths_set = exclude
        .into_iter()
        .map(|p| p.as_ref().canonicalize())
        .collect::<io::Result<BTreeSet<_>>>()?;

    let mut found = BTreeSet::new();

    for path in include_paths_set {
        let sub_result = check_path(&path, &exclude_paths_set)?;
        found.extend(sub_result);
    }

    Ok(found)
}

/// Find files recursively in certain directory.
/// Assert that all paths from input are canonical.
/// Absolute paths are returned.
///
/// the `path` can be either a file or a directory.
/// If `path` is a file, it will be added to the result, which
/// means a BTreeSet with a single element.
/// If `path` is a directory, all files except those in `exclude` will be added.
pub fn check_path(
    path: impl AsRef<Path>,
    exclude: &BTreeSet<PathBuf>,
) -> io::Result<BTreeSet<PathBuf>> {
    let mut result = BTreeSet::new();

    if path.as_ref().is_file() {
        let canonical = path.as_ref().canonicalize()?;
        if is_excluded(&path, &exclude) {
            log::trace!("Excluded file: {:?}", path.as_ref());
        } else if result.insert(canonical) {
            log::trace!("Add file: {:?}", path.as_ref());
        } else {
            panic!("Duplicate file: {:?}. It's impossible!", path.as_ref());
        }
    } else if path.as_ref().is_dir() {
        let entries = fs::read_dir(path.as_ref())?;

        for entry in entries {
            let entry = entry?;
            let path = entry.path().canonicalize()?;
            if is_excluded(&path, exclude) {
                log::trace!("Exclude path: {:?}", path);
            } else {
                let sub_result = check_path(&path, exclude)?;
                result.extend(sub_result);
            }
        }
    } else {
        panic!("Unexpected path type: {:?}", path.as_ref());
    }

    // walk_dir(path.as_ref(), &exclude_paths, &mut result);
    Ok(result)
}

/// Check if a path should be excluded.
#[inline(always)]
fn is_excluded(path: impl AsRef<Path>, exclude_paths: &BTreeSet<PathBuf>) -> bool {
    // Get canonical (absolute) path for the file being checked
    let canonical_path = path
        .as_ref()
        .canonicalize()
        .expect("Failed to canonicalize {path}");

    exclude_paths.iter().any(|exclude| {
        // Try to get canonical path for exclude pattern
        match exclude.canonicalize() {
            Ok(canonical_exclude) => {
                // Compare canonical paths
                canonical_path.starts_with(&canonical_exclude)
                    || canonical_path == canonical_exclude
            }
            Err(e) => {
                log::warn!("Failed to canonicalize exclude path {:?}: {}", exclude, e);
                false
            }
        }
    })
}

/// Normalize a canonical path relative to a target path.
/// Returns the relative path from `target` to `p`.
/// Handles cases where `p` is in a parent directory of `target`.
pub fn normalize_path(p: impl AsRef<Path>, target: impl AsRef<Path>) -> PathBuf {
    let p_canonical = p.as_ref().canonicalize().unwrap();
    let target_canonical = target.as_ref().canonicalize().unwrap();

    // 获取两个路径的组件
    let p_components: Vec<_> = p_canonical.components().collect();
    let target_components: Vec<_> = target_canonical.components().collect();

    // 找到共同前缀的长度
    let mut common_len = 0;
    for (p_comp, target_comp) in p_components.iter().zip(target_components.iter()) {
        if p_comp == target_comp {
            common_len += 1;
        } else {
            break;
        }
    }

    // 计算需要向上多少级（从 target 到共同前缀）
    let up_count = target_components.len() - common_len;

    // 构建相对路径
    let mut result = PathBuf::new();

    // 添加 "../" 部分（向上回溯）
    for _ in 0..up_count {
        result.push("..");
    }

    // 添加从共同前缀到 p 的路径
    for component in p_components.iter().skip(common_len) {
        result.push(component);
    }

    // 如果结果为空，返回 "."
    let result = if result.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        result
    };

    // Windows 平台：将反斜杠替换为斜杠，确保跨平台兼容
    #[cfg(target_os = "windows")]
    {
        let result_str = result.to_string_lossy().replace('\\', "/");
        PathBuf::from(result_str)
    }

    #[cfg(not(target_os = "windows"))]
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// 测试 p 在 target 的子目录中
    #[test]
    fn test_normalize_path_subdirectory() {
        // 创建临时目录结构
        let temp_dir = std::env::temp_dir().join("test_normalize_subdirectory");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let target_dir = temp_dir.join("target");
        let p_file = target_dir.join("src").join("main.rs");
        fs::create_dir_all(p_file.parent().unwrap()).unwrap();
        fs::File::create(&p_file).unwrap();

        let result = normalize_path(&p_file, &target_dir);
        assert_eq!(result, PathBuf::from("src/main.rs"));

        // 清理
        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// 测试 p 在 target 的上级目录中
    #[test]
    fn test_normalize_path_parent_directory() {
        // 创建临时目录结构
        let temp_dir = std::env::temp_dir().join("test_normalize_parent");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let target_dir = temp_dir.join("project").join("build");
        let p_file = temp_dir.join("project").join("main.rs");
        fs::create_dir_all(&target_dir).unwrap();
        fs::File::create(&p_file).unwrap();

        let result = normalize_path(&p_file, &target_dir);
        assert_eq!(result, PathBuf::from("../main.rs"));

        // 清理
        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// 测试 p 在 target 的多级上级目录中
    #[test]
    fn test_normalize_path_multi_level_parent() {
        // 创建临时目录结构
        let temp_dir = std::env::temp_dir().join("test_normalize_multi");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let target_dir = temp_dir.join("project").join("src").join("deep");
        let p_file = temp_dir.join("project").join("file.txt");
        fs::create_dir_all(&target_dir).unwrap();
        fs::File::create(&p_file).unwrap();

        let result = normalize_path(&p_file, &target_dir);
        assert_eq!(result, PathBuf::from("../../file.txt"));

        // 清理
        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// 测试 p 就是 target 本身
    #[test]
    fn test_normalize_path_same_path() {
        // 创建临时目录结构
        let temp_dir = std::env::temp_dir().join("test_normalize_same");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let target_dir = temp_dir.clone();

        let result = normalize_path(&temp_dir, &target_dir);
        assert_eq!(result, PathBuf::from("."));

        // 清理
        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// 测试 p 在 target 的同级目录中
    #[test]
    fn test_normalize_path_sibling_directory() {
        // 创建临时目录结构
        let temp_dir = std::env::temp_dir().join("test_normalize_sibling");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let target_dir = temp_dir.join("project").join("src");
        let p_file = temp_dir.join("project").join("build").join("output.o");
        fs::create_dir_all(&target_dir).unwrap();
        fs::create_dir_all(p_file.parent().unwrap()).unwrap();
        fs::File::create(&p_file).unwrap();

        let result = normalize_path(&p_file, &target_dir);
        assert_eq!(result, PathBuf::from("../build/output.o"));

        // 清理
        let _ = fs::remove_dir_all(&temp_dir);
    }

    /// 测试 p 是文件，target 是目录
    #[test]
    fn test_normalize_path_file_and_directory() {
        // 创建临时目录结构
        let temp_dir = std::env::temp_dir().join("test_normalize_file_dir");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let target_dir = temp_dir.join("project");
        let p_file = temp_dir.join("project").join("README.md");
        fs::create_dir_all(&target_dir).unwrap();
        fs::File::create(&p_file).unwrap();

        let result = normalize_path(&p_file, &target_dir);
        assert_eq!(result, PathBuf::from("README.md"));

        // 清理
        let _ = fs::remove_dir_all(&temp_dir);
    }
}
