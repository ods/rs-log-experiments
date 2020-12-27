use std::ffi::{OsStr, OsString};

#[derive(Debug, PartialEq, thiserror::Error)]
#[error("Invalid value of {name:?}: {value:?}")]
pub struct InvalidEnvValue {
    name: OsString,
    value: OsString,
}

pub fn get<N: AsRef<OsStr>>(
    name: N,
) -> Result<Option<String>, InvalidEnvValue> {
    Ok(std::env::var_os(&name)
        .map(OsString::into_string)
        .transpose()
        .map_err(|err| InvalidEnvValue {
            name: name.as_ref().into(),
            value: err.into(),
        })?
        .filter(|value| !value.is_empty()))
}

#[cfg(test)]
mod test {

    #[test]
    fn test_missing() {
        std::env::remove_var("TEST_MISSING");
        let res = super::get("TEST_MISSING");
        assert_eq!(res, Ok(None));
    }

    #[test]
    fn test_empty() {
        std::env::set_var("TEST_EMPTY", "");
        let res = super::get("TEST_EMPTY");
        assert_eq!(res, Ok(None));
    }

    #[test]
    fn test_normal() {
        std::env::set_var("TEST_NORMAL", "value");
        let res = super::get("TEST_NORMAL");
        assert_eq!(res, Ok(Some("value".into())));
    }

    #[cfg(unix)]
    #[test]
    fn test_error() {
        use std::{ffi::OsStr, os::unix::ffi::OsStrExt};

        let value = OsStr::from_bytes(b"\x66\x6f\x80\x6f").to_os_string();
        std::env::set_var("TEST_ERROR", &value);

        let res = super::get("TEST_ERROR");
        assert_eq!(
            res,
            Err(super::InvalidEnvValue {
                name: "TEST_ERROR".into(),
                value,
            })
        );

        let formatted = format!("{}", res.unwrap_err());
        assert_eq!(formatted, r#"Invalid value of "TEST_ERROR": "fo\x80o""#);
    }
}
