use std::borrow::Cow;

#[derive(Debug, Clone)]
enum Val {
    Var(Variable),
    Expr(Expression),
}

#[derive(Debug, Clone)]
enum Operator {
    And,
    Or,
}

#[derive(Debug, Clone, Default)]
struct Variable {
    pub name: String,
    pub value: bool,
    pub not: bool,
}
impl PartialEq for Variable {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

#[derive(Debug, Clone)]
struct Expression {
    pub raw: String,
    pub lhs: Box<Val>,
    pub rhs: Box<Val>,
    pub op: Operator,
    pub not: bool,
}
impl Expression {
    pub fn evaluate(&self) -> bool {
        let out = match self.op {
            Operator::And => {
                let (lhs, rhs) = self.get_values();
                lhs && rhs
            }
            Operator::Or => {
                let (lhs, rhs) = self.get_values();
                lhs || rhs
            }
        };
        out ^ self.not
    }

    fn get_values(&self) -> (bool, bool) {
        match *self.lhs.clone() {
            Val::Var(lhs_raw) => match *self.rhs.clone() {
                Val::Var(rhs_raw) => {
                    let lhs = if lhs_raw.not {
                        !lhs_raw.value
                    } else {
                        lhs_raw.value
                    };
                    let rhs = if rhs_raw.not {
                        !rhs_raw.value
                    } else {
                        rhs_raw.value
                    };
                    (lhs, rhs)
                }
                Val::Expr(rhs_expr) => {
                    let rhs = rhs_expr.evaluate();
                    let lhs = if lhs_raw.not {
                        !lhs_raw.value
                    } else {
                        lhs_raw.value
                    };
                    (lhs, rhs)
                }
            },
            Val::Expr(lhs_expr) => {
                let lhs = lhs_expr.evaluate();
                match *self.rhs.clone() {
                    Val::Var(rhs_raw) => {
                        let rhs = if rhs_raw.not {
                            !rhs_raw.value
                        } else {
                            rhs_raw.value
                        };
                        (lhs, rhs)
                    }
                    Val::Expr(rhs_expr) => {
                        let rhs = rhs_expr.evaluate();
                        (lhs, rhs)
                    }
                }
            }
        }
    }
}

fn main() {
    println!("{:?}", parse(get_input()));
}

fn get_input() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn parse(input: String) -> Result<Expression, Cow<'static, str>> {
    let mut index: usize = 0;
    let chars = input.chars().into_iter().collect::<Vec<char>>();

    let mut side: bool = false;
    let mut lhs: Val = Val::Var(Variable::default());
    let mut rhs: Val = Val::Var(Variable::default());
    let mut op: Operator = Operator::And;
    let mut not: bool = false;

    const operators: [char; 6] = ['(', ' ', '&', '|', '!', ')'];

    'top: while index < chars.len() {
        let current_char = match chars.get(index) {
            Some(c) => *c,
            None => {
                let e = format!("Logical error when parsing. Index went out of bounds! (Index : {}; Input Size : {}", index, chars.len());
                return Err(Cow::Owned(e));
            }
        };

        match current_char {
            '(' => {
                index += 1;
                let mut expr_str = String::new();
                let mut working_char = match chars.get(index) {
                    Some(c) => *c,
                    None => return Err(Cow::Owned("Malformed statement. Had an opening parenthesis at the end of the statement.".to_owned())),
                };
                while working_char != ')' {
                    expr_str.push(working_char);
                    index += 1;
                    working_char = match chars.get(index) {
                        Some(c) => *c,
                        None => return Err(Cow::Owned("Malformed Statement. Had an opening parenthesis without a paired closing parenthesis.".to_owned())),
                    };
                }
                if !side {
                    lhs = match parse(expr_str) {
                        Ok(e) => Val::Expr({
                            let mut working = e;
                            working.not = not;
                            not = false;
                            working
                        }),
                        Err(e) => return Err(e),
                    };
                    side = true;
                } else {
                    rhs = match parse(expr_str) {
                        Ok(e) => Val::Expr({
                            let mut working = e;
                            working.not = not;
                            working
                        }),
                        Err(e) => return Err(e),
                    };
                    break;
                }
            }
            ' ' => {}
            '&' | '|' => {
                if !side {
                    return Err(Cow::Owned("Malformed Statement. Had an operator before left hand side of expression.".to_owned()));
                }
                match current_char {
                    '&' => {
                        op = Operator::And;
                    },
                    '|' => {
                        op = Operator::Or;
                    },
                    _ => return Err(Cow::Owned(format!("Logic error. Apparently {} matches to either ['&' or '|'], but also doesn't. Thanks Rust!", current_char)))
                }
            }
            '!' => {
                not = true;
            }
            ')' => return Err(Cow::Owned(
                "Malformed Statement. Had a closing parenthesis without an opening parenthesis."
                    .to_owned(),
            )),
            _ => {
                let mut var_name = String::new();
                var_name.push(current_char);
                index += 1;
                let mut working_char = match chars.get(index) {
                    Some(c) => *c,
                    None => {
                        if !side {
                            return Err(Cow::Owned(format!("Malformed statement. Expected variable {} to be the rhs value, but was the lhs.", var_name)));
                        }
                        rhs = Val::Var({
                            let working = Variable {
                                name: var_name,
                                value: false,
                                not: not,
                            };
                            working
                        });
                        break;
                    }
                };
                while !operators.contains(&working_char) {
                    var_name.push(working_char);
                    index += 1;
                    working_char = match chars.get(index) {
                        Some(c) => *c,
                        None => break,
                    };
                }
                if !side {
                    lhs = Val::Var({
                        let working = Variable {
                            name: var_name,
                            value: false,
                            not: not,
                        };
                        not = false;
                        working
                    });
                    side = true;
                    index += 1;
                    continue 'top;
                } else {
                    rhs = Val::Var({
                        let working = Variable {
                            name: var_name,
                            value: false,
                            not: not,
                        };
                        working
                    });
                    break 'top;
                }
            }
        }
        index += 1;
    }

    Ok(Expression {
        raw: input,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
        op: op,
        not: false,
    })
}
