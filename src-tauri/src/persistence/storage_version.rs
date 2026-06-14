use crate::persistence::models::StorageVersion;

/// 版本兼容性判断结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionCompatibility {
    /// major 一致且 minor >= 记录值，可加载。
    Compatible,
    /// major 不一致或 minor < 记录值，拒绝加载。
    Incompatible,
}

/// StorageVersion 的校验与比较服务。
pub struct StorageVersionService;

impl StorageVersionService {
    /// 判断记录的存储版本是否与当前版本兼容。
    ///
    /// 规则：
    /// - `major` 一致且 `minor >=` 记录值 → Compatible
    /// - 否则 → Incompatible
    pub fn check_compatibility(recorded: &StorageVersion) -> VersionCompatibility {
        if recorded.major != StorageVersion::CURRENT.major {
            return VersionCompatibility::Incompatible;
        }
        if recorded.minor > StorageVersion::CURRENT.minor {
            return VersionCompatibility::Incompatible;
        }
        VersionCompatibility::Compatible
    }

    /// 判断当前版本是否能以向后兼容方式写入旧 minor 版本。
    ///
    /// MVP 阶段仅支持完全相同的版本号写入。
    pub fn can_write_as(recorded: &StorageVersion) -> bool {
        *recorded == StorageVersion::CURRENT
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_version_is_compatible_with_itself() {
        let result = StorageVersionService::check_compatibility(&StorageVersion::CURRENT);
        assert_eq!(result, VersionCompatibility::Compatible);
    }

    #[test]
    fn same_major_equal_minor_is_compatible() {
        let recorded = StorageVersion {
            major: 1,
            minor: 0,
            patch: 5,
        };
        assert_eq!(
            StorageVersionService::check_compatibility(&recorded),
            VersionCompatibility::Compatible
        );
    }

    #[test]
    fn same_major_lower_minor_is_compatible() {
        let recorded = StorageVersion {
            major: 1,
            minor: 0,
            patch: 0,
        };
        assert_eq!(
            StorageVersionService::check_compatibility(&recorded),
            VersionCompatibility::Compatible
        );
    }

    #[test]
    fn same_major_higher_minor_is_incompatible() {
        let recorded = StorageVersion {
            major: 1,
            minor: 1,
            patch: 0,
        };
        assert_eq!(
            StorageVersionService::check_compatibility(&recorded),
            VersionCompatibility::Incompatible
        );
    }

    #[test]
    fn different_major_is_incompatible() {
        let recorded = StorageVersion {
            major: 2,
            minor: 0,
            patch: 0,
        };
        assert_eq!(
            StorageVersionService::check_compatibility(&recorded),
            VersionCompatibility::Incompatible
        );
    }

    #[test]
    fn zero_major_is_incompatible() {
        let recorded = StorageVersion {
            major: 0,
            minor: 9,
            patch: 9,
        };
        assert_eq!(
            StorageVersionService::check_compatibility(&recorded),
            VersionCompatibility::Incompatible
        );
    }

    #[test]
    fn can_write_as_requires_exact_current() {
        assert!(StorageVersionService::can_write_as(&StorageVersion::CURRENT));
        assert!(!StorageVersionService::can_write_as(&StorageVersion {
            major: 1,
            minor: 0,
            patch: 1,
        }));
        assert!(!StorageVersionService::can_write_as(&StorageVersion {
            major: 1,
            minor: 1,
            patch: 0,
        }));
    }
}
