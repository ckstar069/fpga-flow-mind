use std::path::Path;

use crate::models::enums::{Language, SourceKind};

/// 对单个文件进行类型识别，返回 (language, source_kind)。
///
/// 分类优先级：测试文件名模式优先于扩展名。
/// 不读取文件内容，仅基于路径元数据。
pub fn classify_file(path: &Path) -> (Language, SourceKind) {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_lowercase();
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    // 测试文件模式优先
    let is_test = name.starts_with("test_")
        || name.ends_with("_test")
        || name.ends_with("_tb")
        || name.contains("/tests/");

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
pub fn is_binary(path: &Path) -> bool {
    const SAMPLE_SIZE: usize = 8192;
    const NUL_THRESHOLD: f64 = 0.10;

    let content = match std::fs::read(path) {
        Ok(c) => c,
        Err(_) => return true,
    };

    let sample = &content[..content.len().min(SAMPLE_SIZE)];
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
}
