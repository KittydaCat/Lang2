use std::collections::HashMap;

use crate::parser::{RawType, RawTypeDefinition, Value};

#[derive(Clone, Debug, Hash)]
pub enum Type {
    Unit,
    Int,

    Function(Box<Type>, Box<Type>),

    TypeDef(String),

    Unknown(usize),
}

#[derive(Debug)]
pub struct TypedValue {
    val_type: Type,
    val_source: ValueSource,
}

#[derive(Debug)]
enum ValueSource {
    FunctionDefinition(String, Box<TypedValue>),

    FunctionCall(Box<TypedValue>, Box<TypedValue>),

    Constant(String),

    Number(f64),
    Unit,
}

#[derive(Debug)]
pub enum TypedDefinition {
    TypeDefinition(TypeDefinition),
    ConstantDefinition(String, TypedValue),
}

#[derive(Debug)]
pub struct TypeDefinition {
    name: String,
    variants: Vec<(String, Type)>,
}

struct State {
    const_types: HashMap<String, Type>,
    unknown_types: HashMap<usize, Type>,
    curr: usize,
}

impl State {
    fn new(const_types: HashMap<String, Type>) -> Self {
        Self {
            const_types,
            unknown_types: HashMap::new(),
            curr: 0,
        }
    }

    fn get_type(&self, str: &str) -> Type {
        self.const_types.get(str).unwrap().clone()
    }

    fn add_temp_type(&mut self, const_type: Type, name: String) -> TempType {
        assert!(self.const_types.insert(name, const_type).is_none());

        TempType
    }

    fn remove_temp_type(&mut self, _: TempType, name: &str) {
        assert!(self.const_types.remove(name).is_some());
    }

    fn add_unknown(&mut self) -> Type {
        let id = self.curr;
        self.curr += 1;

        Type::Unknown(id)
    }

    // TODO fix this shit
    fn matches(&mut self, type1: &Type, type2: &Type) -> bool {
        match (type1, type2) {
            (Type::Unit, Type::Unit) => true,
            (Type::Int, Type::Int) => true,
            (Type::TypeDef(x), Type::TypeDef(y)) => x == y,

            (Type::Function(a, b), Type::Function(c, d)) => self.matches(a, c) & self.matches(b, d),

            // TODO
            (Type::Unknown(a), Type::Unknown(b)) => {
                match (self.unknown_types.get(a), self.unknown_types.get(b)) {
                    (None, None) => {
                        let max = std::cmp::max(a, b);
                        let min = std::cmp::min(a, b);

                        assert!(
                            self.unknown_types
                                .insert(*max, Type::Unknown(*min))
                                .is_none()
                        );

                        true
                    }
                    (None, Some(typ)) => {
                        assert!(self.unknown_types.insert(*a, typ.clone()).is_none());
                        true
                    }
                    (Some(typ), None) => {
                        assert!(self.unknown_types.insert(*b, typ.clone()).is_none());
                        true
                    }
                    (Some(t1), Some(t2)) => self.matches(&t1.clone(), &t2.clone()),
                }
            }

            (typ, Type::Unknown(id)) | (Type::Unknown(id), typ) => {
                if let Some(typ2) = self.unknown_types.get(id) {
                    self.matches(typ, &typ2.clone())
                } else {
                    assert!(self.unknown_types.insert(*id, typ.clone()).is_none());

                    true
                }
            }

            _ => false,
        }
    }

    fn instantiate_val(&self, val: &mut TypedValue, unk: usize) {
        self.instantiate_type(&mut val.val_type, unk);

        match &mut val.val_source {
            ValueSource::FunctionDefinition(_, typed_value) => {
                self.instantiate_val(typed_value, unk);
            }
            ValueSource::FunctionCall(typed_value, typed_value1) => {
                self.instantiate_val(typed_value, unk);
                self.instantiate_val(typed_value1, unk);
            }
            _ => {}
        }
    }

    fn instantiate_type(&self, typ: &mut Type, unk: usize) {
        match typ {
            Type::Function(ty1, ty2) => {
                self.instantiate_type(ty1, unk);
                self.instantiate_type(ty2, unk);
            }

            unknown @ Type::Unknown(_) => {
                let Type::Unknown(id) = unknown else {
                    unimplemented!()
                };

                let id = *id;

                if let Some(typ) = self.unknown_types.get(&id) {
                    *unknown = typ.clone();
                } else {
                    assert!(id < unk)
                }
            }
            _ => {}
        }
    }

    fn remove_unknown(&mut self, id: usize) {
        while self.curr > id {
            self.curr -= 1;

            assert!(self.unknown_types.remove(&self.curr).is_some());
        }
    }
}

#[must_use]
struct TempType;

// #[must_use]
// struct Unknown;

pub fn type_definition(
    definitions: (Vec<RawTypeDefinition>, Vec<(String, Value, RawType)>),
) -> (Vec<TypeDefinition>, Vec<(String, TypedValue)>) {
    let type_def = definitions.0;

    let const_def = definitions.1;

    let const_types = const_def
        .iter()
        .map(|x| (x.0.clone(), raw_type_to_type(x.2.clone())))
        .collect::<HashMap<String, Type>>();

    let mut state = State::new(const_types);

    (
        type_def.into_iter().map(parse_type_def).collect(),
        const_def
            .into_iter()
            .map(|x| (x.0, parse_val(x.1, raw_type_to_type(x.2), &mut state)))
            .collect(),
    )
}

fn parse_type_def(type_def: RawTypeDefinition) -> TypeDefinition {
    let variants = type_def
        .variants
        .into_iter()
        .map(|x| (x.0, raw_type_to_type(x.1)))
        .collect();

    TypeDefinition {
        name: type_def.name,
        variants,
    }
}

fn parse_val(value: Value, goal: Type, state: &mut State) -> TypedValue {
    match value {
        Value::FunctionDefinition(str, typ, val) => {
            let Type::Function(inp, out) = goal.clone() else {
                unimplemented!()
            };

            let temp = state.add_temp_type(*inp, str.clone());

            let typ_val = parse_val(val.as_ref().clone(), *out, state);

            state.remove_temp_type(temp, &str);

            TypedValue {
                val_type: goal,
                val_source: ValueSource::FunctionDefinition(str, Box::new(typ_val)),
            }
        }

        Value::FunctionCall(func, arg) => {
            let unk = state.add_unknown();

            let mut parsed_arg = parse_val(*arg, unk.clone(), state);

            let Type::Unknown(id) = unk else {
                unimplemented!()
            };

            let mut parsed_func = parse_val(
                *func,
                Type::Function(Box::new(unk.clone()), Box::new(goal.clone())),
                state,
            );

            assert!(state.unknown_types.get(&id).is_some());

            state.instantiate_val(&mut parsed_func, id);
            state.instantiate_val(&mut parsed_arg, id);

            state.remove_unknown(id);

            TypedValue {
                val_type: goal,
                val_source: ValueSource::FunctionCall(Box::new(parsed_func), Box::new(parsed_arg)),
            }
        }

        Value::Constant(str) => {
            assert!(state.matches(&goal, &state.get_type(&str)));

            TypedValue {
                val_type: state.get_type(&str),
                val_source: ValueSource::Constant(str),
            }
        }
        Value::Number(x) => {
            assert!(state.matches(&goal, &Type::Int));

            TypedValue {
                val_type: Type::Int,
                val_source: ValueSource::Number(x),
            }
        }
        Value::Unit => {
            assert!(state.matches(&goal, &Type::Unit));

            TypedValue {
                val_type: Type::Unit,
                val_source: ValueSource::Unit,
            }
        }

        Value::Match(_, _) => todo!(),
    }
}

fn raw_type_to_type(t: RawType) -> Type {
    match t {
        RawType::Unit => Type::Unit,
        RawType::Function(raw_type, raw_type1) => Type::Function(
            Box::from(raw_type_to_type(*raw_type)),
            Box::from(raw_type_to_type(*raw_type1)),
        ),
        RawType::TypeDef(x) => Type::TypeDef(x),
    }
}
