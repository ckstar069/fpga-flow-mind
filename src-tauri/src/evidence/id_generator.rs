/// evidence_id 生成器
///
/// 格式: "EV-<stage_id>-<6位序号>"
/// 单次 `collect_evidence` 调用内唯一，不做跨会话持久化
pub struct EvidenceIdGenerator {
    stage_id: String,
    counter: u32,
}

impl EvidenceIdGenerator {
    pub fn new(stage_id: &str) -> Self {
        Self {
            stage_id: stage_id.to_string(),
            counter: 0,
        }
    }

    /// 生成下一个唯一 evidence_id
    /// 格式: "EV-<stage_id>-<6位序号>"
    /// counter 从 1 开始，每次调用递增
    pub fn next_id(&mut self) -> String {
        self.counter += 1;
        format!("EV-{}-{:06}", self.stage_id, self.counter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_01_first_id_format() {
        let mut gen = EvidenceIdGenerator::new("L0");
        assert_eq!(gen.next_id(), "EV-L0-000001");
    }

    #[test]
    fn id_02_sequential_increment() {
        let mut gen = EvidenceIdGenerator::new("L0");
        assert_eq!(gen.next_id(), "EV-L0-000001");
        assert_eq!(gen.next_id(), "EV-L0-000002");
        assert_eq!(gen.next_id(), "EV-L0-000003");
    }

    #[test]
    fn id_03_different_stage() {
        let mut gen = EvidenceIdGenerator::new("RTL");
        assert_eq!(gen.next_id(), "EV-RTL-000001");
    }

    #[test]
    fn id_04_uniqueness_1000() {
        let mut gen = EvidenceIdGenerator::new("L0");
        let mut ids = std::collections::HashSet::new();
        for _ in 0..1000 {
            let id = gen.next_id();
            assert!(ids.insert(id), "duplicate id generated");
        }
        assert_eq!(ids.len(), 1000);
    }

    #[test]
    fn id_stage_id_preserved_verbatim() {
        let mut gen = EvidenceIdGenerator::new("constraints");
        assert_eq!(gen.next_id(), "EV-constraints-000001");
    }
}
