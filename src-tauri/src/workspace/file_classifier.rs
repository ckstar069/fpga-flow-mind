use std::path::Path;

use crate::models::enums::{Language, SourceKind};

/// 对单个文件进行类型识别，返回 (language, source_kind)。
///
/// 分类优先级：测试文件名模式优先于扩展名。
/// 不读取文件内容，仅基于路径元数据。
pub fn classify_file(path: &Path) -> (Language, SourceKind) {
    let stem = path
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // 测试文件模式优先（基于文件名 stem 或路径中的 tests/ 目录）
    let path_str = path.to_string_lossy().to_lowercase();
    let is_test = stem.starts_with("test_")
        || stem.ends_with("_test")
        || stem.ends_with("_tb")
        || path_str.contains("/tests/");

    if is_test {
        match ext.as_str() {
            "py" => return (Language::Python, SourceKind::Test),
            "v" => return (Language::Verilog, SourceKind::Test),
            "sv" => return (Language::SystemVerilog, SourceKind::Test),
            _ => {}
        }
    }

    match ext.as_str() {
        "py" => (Language::Python, SourceKind::PythonStage),
        "v" | "vh" => (Language::Verilog, SourceKind::Rtl),
        "sv" => (Language::SystemVerilog, SourceKind::Rtl),
        "md" => (Language::Markdown, SourceKind::Doc),
        "rst" | "txt" => (Language::Text, SourceKind::Doc),
        "json" => (Language::Json, SourceKind::Config),
        "yaml" | "yml" => (Language::Yaml, SourceKind::Config),
        "toml" => (Language::Toml, SourceKind::Config),
        _ => (Language::Unknown, SourceKind::Config),
    }
}

/// 判断文件是否为二进制（通过检查前 8KB 中 NUL 字节比例 > 10%）。
/// 对不可读文件返回 `true`（保守跳过）。
/// 只读取前 8KB，不加载整文件。
pub fn is_binary(path: &Path) -> bool {
    const SAMPLE_SIZE: usize = 8192;
    const NUL_THRESHOLD: f64 = 0.10;

    let mut file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return true,
    };

    use std::io::Read;
    let mut buf = vec![0u8; SAMPLE_SIZE];
    let n = match file.read(&mut buf) {
        Ok(n) => n,
        Err(_) => return true,
    };

    let sample = &buf[..n];
    if sample.is_empty() {
        return false;
    }

    let nul_count = sample.iter().filter(|&&b| b == 0u8).count();
    (nul_count as f64) / (sample.len() as f64) > NUL_THRESHOLD
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn py_file_is_python_stage() {
        let path = Path::new("/project/L0/adder.py");
        let (lang, kind) = classify_file(path);
        assert_eq!(lang, Language::Python);
        assert_eq!(kind, SourceKind::PythonStage);
    }

    #[test]
    fn test_py_file_is_test() {
        let path = Path::new("/project/L0/test_adder.py");
        let (lang, kind) = classify_file(path);
        assert_eq!(lang, Language::Python);
        assert_eq!(kind, SourceKind::Test);
    }

    #[test]
    fn v_file_is_rtl() {
        let path = Path::new("/project/RTL/top.v");
        let (lang, kind) = classify_file(path);
        assert_eq!(lang, Language::Verilog);
        assert_eq!(kind, SourceKind::Rtl);
    }

    #[test]
    fn sv_file_is_rtl() {
        let path = Path::new("/project/RTL/top.sv");
        let (lang, kind) = classify_file(path);
        assert_eq!(lang, Language::SystemVerilog);
        assert_eq!(kind, SourceKind::Rtl);
    }

    #[test]
    fn md_file_is_doc() {
        let path = Path::new("/project/README.md");
        let (lang, kind) = classify_file(path);
        assert_eq!(lang, Language::Markdown);
        assert_eq!(kind, SourceKind::Doc);
    }

    #[test]
    fn binary_file_detected() {
        let tmp = tempfile::tempdir().unwrap();
        let bin_path = tmp.path().join("data.bin");
        let mut data = vec![0u8; 1024];
        data[0] = 0x89;
        data[1] = 0x50;
        fs::write(&bin_path, data).unwrap();
        assert!(is_binary(&bin_path));
    }

    #[test]
    fn text_file_not_binary() {
        let tmp = tempfile::tempdir().unwrap();
        let txt_path = tmp.path().join("hello.txt");
        fs::write(&txt_path, b"Hello, World!\n").unwrap();
        assert!(!is_binary(&txt_path));
    }

    #[test]
    fn big_file_binary_checks_only_head() {
        let tmp = tempfile::tempdir().unwrap();
        let big = tmp.path().join("big.v");
        // 前 8192 字节是文本，后面全 NUL —— 只读前 8KB，应判定为文本
        let mut data = vec![b'a'; 8192];
        data.extend(vec![0u8; 6 * 1024 * 1024]);
        fs::write(&big, data).unwrap();
        assert!(!is_binary(&big), "只读前 8KB 时应判定为文本");
    }

    #[test]
    fn suffix_test_py_is_test() {
        let path = Path::new("/project/L0/foo_test.py");
        let (lang, kind) = classify_file(path);
        assert_eq!(lang, Language::Python);
        assert_eq!(kind, SourceKind::Test);
    }

    #[test]
    fn suffix_tb_v_is_test() {
        let path = Path::new("/project/RTL/top_tb.v");
        let (lang, kind) = classify_file(path);
        assert_eq!(lang, Language::Verilog);
        assert_eq!(kind, SourceKind::Test);
    }

    #[test]
    fn tests_dir_sv_is_test() {
        let path = Path::new("/project/L0/tests/test_stage.sv");
        let (lang, kind) = classify_file(path);
        assert_eq!(lang, Language::SystemVerilog);
        assert_eq!(kind, SourceKind::Test);
    }
}
