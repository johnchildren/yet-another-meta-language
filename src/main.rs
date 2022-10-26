use serde_yaml::{Mapping, Value};

fn eval(env: &mut Mapping, val: &Value) -> Value {
    match val {
        v @ Value::Null | v @ Value::Bool(_) | v @ Value::Number(_) | v @ Value::String(_) => {
            let mut u = v;
            while let Some(v) = env.get(u) {
                u = v;
            }
            u.clone()
        }
        Value::Sequence(vec) => {
            Value::Sequence(vec.iter().map(|v| eval(env, v)).collect::<Vec<Value>>())
        }
        Value::Mapping(map) => {
            let new_map = map
                .iter()
                .map(|(k, v)| (k.clone(), eval(env, v)))
                .collect::<Mapping>();

            new_map.iter().for_each(|(k, v)| {
                env.insert(k.clone(), v.clone());
            });

            Value::Mapping(new_map)
        }
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_yaml::Number;

    #[test]
    fn atoms() {
        assert_eq!(eval(&mut Mapping::new(), &Value::Null), Value::Null);
        assert_eq!(
            eval(&mut Mapping::new(), &Value::Bool(true)),
            Value::Bool(true)
        );
        assert_eq!(
            eval(&mut Mapping::new(), &Value::Number(Number::from(3))),
            Value::Number(Number::from(3))
        );
        assert_eq!(
            eval(&mut Mapping::new(), &Value::String("foo".to_string())),
            Value::String("foo".to_string())
        );
    }

    #[test]
    fn invariance_in_empty_env() {
        assert_eq!(
            eval(
                &mut Mapping::new(),
                &Value::Sequence(vec![Value::String("foo".to_string())])
            ),
            Value::Sequence(vec![Value::String("foo".to_string())])
        );
    }

    #[test]
    fn reading_from_environment() {
        let mut env = Mapping::new();
        env.insert(
            Value::String("foo".to_string()),
            Value::String("bar".to_string()),
        );
        assert_eq!(
            eval(
                &mut env,
                &Value::Sequence(vec![Value::String("foo".to_string())])
            ),
            Value::Sequence(vec![Value::String("bar".to_string())])
        );
    }

    #[test]
    fn reading_from_previous_statements() {
        let mut program = Mapping::new();
        program.insert(
            Value::String("foo".to_string()),
            Value::String("bar".to_string()),
        );
        program.insert(
            Value::String("bar".to_string()),
            Value::String("baz".to_string()),
        );

        assert_eq!(
            eval(
                &mut Mapping::new(),
                &mut Value::Sequence(vec![
                    Value::Mapping(program.clone()),
                    Value::String("foo".to_string())
                ])
            ),
            Value::Sequence(vec![
                Value::Mapping(program),
                Value::String("baz".to_string())
            ])
        );
    }

    #[test]
    fn merging_programs() {
        let mut program1 = Mapping::new();
        program1.insert(
            Value::String("foo".to_string()),
            Value::String("bar".to_string()),
        );
        let mut program2 = Mapping::new();
        program2.insert(
            Value::String("bar".to_string()),
            Value::String("baz".to_string()),
        );

        assert_eq!(
            eval(
                &mut Mapping::new(),
                &mut Value::Sequence(vec![
                    Value::Mapping(program1.clone()),
                    Value::Mapping(program2.clone()),
                    Value::String("foo".to_string())
                ])
            ),
            Value::Sequence(vec![
                Value::Mapping(program1),
                Value::Mapping(program2),
                Value::String("baz".to_string())
            ])
        );
    }
}
