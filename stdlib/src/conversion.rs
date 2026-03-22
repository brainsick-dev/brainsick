use ty::types::{Boolean, Variant, Result};

pub fn c_bool(expression: Variant) -> Result<Boolean> {
    match expression {
        Variant::Boolean(boolean) => Ok(boolean),
        Variant::Integer(integer) => if integer.is_zero() {
            Ok(Boolean::r#false())
        } else {
            Ok(Boolean::r#true())
        }
        Variant::Long(long) => if long.is_zero() {
            Ok(Boolean::r#false())
        } else {
            Ok(Boolean::r#true())
        }
    }
}