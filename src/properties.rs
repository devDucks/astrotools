use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
enum PropValue {
    Int(u32),
    Bool(bool),
    Str(String),
    Float(f32),
}

#[derive(Debug, Serialize, Deserialize)]
/// Struct to serialize an update property request coming from MQTT
struct UpdatePropertyRequest {
    prop_name: String,
    value: PropValue,
}

#[derive(Debug, PartialEq)]
pub enum PropertyError {
    CannotUpdateReadOnlyProp,
    InvalidValue,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Permission {
    ReadOnly = 0,
    ReadWrite,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Range<T> {
    min: T,
    max: T,
}

impl<T> Range<T> {
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }

    pub fn max(&self) -> &T {
        &self.max
    }

    pub fn min(&self) -> &T {
        &self.min
    }
}

pub trait Prop<T> {
    fn value(&self) -> &T;
    fn update_allowed(&self) -> Result<(), PropertyError>;
    fn update(&mut self, value: T) -> Result<(), PropertyError>;
    fn update_int(&mut self, value: T) -> Result<(), PropertyError>;
    fn validate(&self, val: &T) -> Result<(), PropertyError>;
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct RangeProperty<T> {
    value: T,
    permission: Permission,
    range: Range<T>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Property<T> {
    value: T,
    permission: Permission,
}

impl<T> Property<T> {
    pub fn new(value: T, permission: Permission) -> Self {
        Self { value, permission }
    }
}

impl<T> Prop<T> for Property<T> {
    fn value(&self) -> &T {
        &self.value
    }

    fn update_allowed(&self) -> Result<(), PropertyError> {
        match self.permission {
            Permission::ReadOnly => Err(PropertyError::CannotUpdateReadOnlyProp),
            _ => Ok(()),
        }
    }

    fn update(&mut self, value: T) -> Result<(), PropertyError> {
        self.update_allowed()?;
        self.value = value;
        Ok(())
    }

    fn update_int(&mut self, value: T) -> Result<(), PropertyError> {
        self.value = value;
        Ok(())
    }

    fn validate(&self, _val: &T) -> Result<(), PropertyError> {
        Ok(())
    }
}

impl<T> Prop<T> for RangeProperty<T> {
    fn value(&self) -> &T {
        &self.value
    }

    fn update_allowed(&self) -> Result<(), PropertyError> {
        match self.permission {
            Permission::ReadOnly => Err(PropertyError::CannotUpdateReadOnlyProp),
            _ => Ok(()),
        }
    }

    fn update(&mut self, value: T) -> Result<(), PropertyError> {
        self.update_allowed()?;
        self.value = value;
        Ok(())
    }

    fn update_int(&mut self, value: T) -> Result<(), PropertyError> {
        self.value = value;
        Ok(())
    }

    fn validate(&self, _val: &T) -> Result<(), PropertyError> {
        Ok(())
    }
}

impl<T> RangeProperty<T> {
    pub fn new(value: T, permission: Permission, min: T, max: T) -> Self {
        Self {
            value,
            permission,
            range: Range::new(min, max),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ChoiceProperty<T>
where
    T: std::clone::Clone,
{
    value: T,
    permission: Permission,
    choices: Vec<T>,
}

impl<T: std::clone::Clone + std::cmp::PartialEq> ChoiceProperty<T> {
    pub fn new(value: T, permission: Permission, choices: Vec<T>) -> Self {
        Self {
            value,
            permission,
            choices,
        }
    }
}

impl<T: std::clone::Clone + std::cmp::PartialEq> Prop<T> for ChoiceProperty<T> {
    fn value(&self) -> &T {
        &self.value
    }

    fn update_allowed(&self) -> Result<(), PropertyError> {
        match self.permission {
            Permission::ReadOnly => Err(PropertyError::CannotUpdateReadOnlyProp),
            _ => Ok(()),
        }
    }

    fn update(&mut self, value: T) -> Result<(), PropertyError> {
        self.update_allowed()?;
        self.validate(&value)?;
        self.value = value;
        Ok(())
    }

    fn update_int(&mut self, value: T) -> Result<(), PropertyError> {
        self.validate(&value)?;
        self.value = value;
        Ok(())
    }

    fn validate(&self, val: &T) -> Result<(), PropertyError> {
        if !self.choices.contains(val) {
            return Err(PropertyError::InvalidValue);
        }
        Ok(())
    }
}

#[cfg(test)]
mod unit_tests {
    use super::{ChoiceProperty, Permission, Prop, Property, PropertyError, RangeProperty};

    #[test]
    fn test_bool_prop_initialization() {
        let p = Property::new(false, Permission::ReadOnly);
        assert_eq!(p.value(), &false);
    }

    #[test]
    fn test_prop_readonly_cannot_be_updated() {
        let mut p = Property::new(false, Permission::ReadOnly);
        let res = p.update(true);
        assert_eq!(res, Err(PropertyError::CannotUpdateReadOnlyProp));
        assert_eq!(p.value(), &false);
    }

    #[test]
    fn test_prop_readwrite_can_be_written() {
        let mut p = Property::new(false, Permission::ReadWrite);
        let res = p.update(true);
        assert_eq!(res, Ok(()));
        assert_eq!(p.value(), &true);
    }

    #[test]
    fn test_u64_prop() {
        let mut p: Property<u64> = Property::new(78, Permission::ReadWrite);
        let _res = p.update(55);
        assert_eq!(p.value(), &55_u64);
    }

    #[test]
    fn test_str_prop() {
        let test_str = String::from("test");
        let p: Property<String> = Property::new(test_str.clone(), Permission::ReadWrite);
        assert_eq!(p.value(), &test_str);
    }

    #[test]
    fn test_float_prop_initialization_no_range() {
        let test_val = 5.32_f64;
        let p: Property<f64> = Property::new(test_val, Permission::ReadOnly);
        assert_eq!(p.value(), &test_val);
    }

    #[test]
    fn test_range_prop() {
        let test_val = 5.32_f64;
        let min_range = 10.0_f64;
        let max_range = 100.0_f64;
        let p = RangeProperty::new(test_val.clone(), Permission::ReadOnly, min_range, max_range);
        assert_eq!(p.range.min(), &min_range);
        assert_eq!(p.range.max(), &max_range);
    }

    #[test]
    fn test_choice_prop() {
        let mut p = ChoiceProperty::new(0, Permission::ReadWrite, vec![0, 1, 2, 3]);
        let _ = p.update(1);
        assert_eq!(p.value(), &1);
        let res = p.update(100);
        assert_eq!(res, Err(PropertyError::InvalidValue));
    }
}

#[cfg(test)]
mod serialization_tests {
    use super::{Permission, Property, RangeProperty};

    #[test]
    fn test_serialize_num_prop() {
        let p = Property::new(5, Permission::ReadOnly);
        assert_eq!(
            serde_json::to_string(&p).unwrap(),
            r#"{"value":5,"permission":"ReadOnly"}"#
        );
    }

    #[test]
    fn test_serialize_str_prop() {
        let p = RangeProperty::new(5, Permission::ReadOnly, -1000, 3000);
        assert_eq!(
            serde_json::to_string(&p).unwrap(),
            r#"{"value":5,"permission":"ReadOnly","range":{"min":-1000,"max":3000}}"#
        );
    }

    #[test]
    fn test_serialize_bool_prop() {
        let p = Property::new(true, Permission::ReadOnly);
        assert_eq!(
            serde_json::to_string(&p).unwrap(),
            r#"{"value":true,"permission":"ReadOnly"}"#
        );
    }
}
