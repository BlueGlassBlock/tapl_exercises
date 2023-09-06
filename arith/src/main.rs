use pest::{
    iterators::{Pair, Pairs},
    Parser,
};
use pest_derive::Parser;
use thiserror::Error;

#[derive(Parser)]
#[grammar = "arith.pest"]
struct ArithParser;

#[derive(Debug, PartialEq)]
enum AST {
    True,
    False,
    Zero,
    Succ(Box<AST>),
    Pred(Box<AST>),
    IsZero(Box<AST>),
    IfThenElse(Box<AST>, Box<AST>, Box<AST>),
}

trait TryTake<T, E> {
    fn try_take(&mut self) -> Result<T, E>;
}

impl<'i> TryTake<Pair<'i, Rule>, ArithError> for Pairs<'i, Rule> {
    fn try_take(&mut self) -> Result<Pair<'i, Rule>, ArithError> {
        self.next().ok_or(ArithError::EmptyPairsError)
    }
}

impl TryFrom<Pair<'_, Rule>> for AST {
    type Error = ArithError;
    fn try_from(value: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        match value.as_rule() {
            Rule::Term => AST::try_from(value.into_inner().try_take()?),
            Rule::True => Ok(AST::True),
            Rule::False => Ok(AST::False),
            Rule::Zero => Ok(AST::Zero),
            Rule::Succ => {
                let mut pairs = value.into_inner();
                Ok(AST::Succ(Box::new(pairs.try_take()?.try_into()?)))
            }
            Rule::Pred => {
                let mut pairs = value.into_inner();
                Ok(AST::Pred(Box::new(pairs.try_take()?.try_into()?)))
            }
            Rule::IsZero => {
                let mut pairs = value.into_inner();
                Ok(AST::IsZero(Box::new(pairs.try_take()?.try_into()?)))
            }
            Rule::IfThenElse => {
                let mut pairs = value.into_inner();
                Ok(AST::IfThenElse(
                    Box::new(pairs.try_take()?.try_into()?),
                    Box::new(pairs.try_take()?.try_into()?),
                    Box::new(pairs.try_take()?.try_into()?),
                ))
            }
            _ => Err(ArithError::UnexpectedNodeError(value.as_rule())),
        }
    }
}

fn is_numeric_val(v: &AST) -> bool {
    match v {
        AST::Zero => true,
        AST::Succ(v) => is_numeric_val(v),
        _ => false,
    }
}

fn is_val(v: &AST) -> bool {
    match v {
        AST::True | AST::False => true,
        v if is_numeric_val(v) => true,
        _ => false,
    }
}

fn eval_ast(v: AST) -> Result<AST, ArithError> {
    match v {
        v if is_val(&v) => Ok(v), // B-Value
        AST::IfThenElse(cond, then, els) => {
            let cond = eval_ast(*cond)?;
            match cond {
                AST::True => eval_ast(*then), // B-IfTrue
                AST::False => eval_ast(*els), // B-IfFalse
                v => Err(ArithError::UnknownRuleError(v)),
            }
        }
        AST::Succ(v) => {
            let v = eval_ast(*v)?;
            match v {
                v if is_numeric_val(&v) => Ok(AST::Succ(Box::new(v))), // B-Succ
                v => Err(ArithError::UnknownRuleError(v)),
            }
        }
        AST::Pred(v) => {
            let v = eval_ast(*v)?;
            match v {
                AST::Zero => Ok(AST::Zero),                    // B-PredZero
                AST::Succ(v) if is_numeric_val(&*v) => Ok(*v), // B-PredSucc
                v => Err(ArithError::UnknownRuleError(v)),
            }
        }
        AST::IsZero(v) => {
            let v = eval_ast(*v)?;
            match v {
                AST::Zero => Ok(AST::True),                           // B-IsZeroZero
                AST::Succ(v) if is_numeric_val(&v) => Ok(AST::False), // B-IsZeroSucc
                v => Err(ArithError::UnknownRuleError(v)),
            }
        }
        v => Err(ArithError::UnknownRuleError(v)),
    }
}

fn arith_size(v: &AST) -> u128 {
    match v {
        AST::True | AST::False | AST::Zero => 1,
        AST::Succ(v) | AST::Pred(v) | AST::IsZero(v) => 1 + arith_size(v),
        AST::IfThenElse(cond, then, els) => {
            1 + arith_size(cond) + arith_size(then) + arith_size(els)
        }
    }
}

fn arith_depth(v: &AST) -> u128 {
    match v {
        AST::True | AST::False | AST::Zero => 1,
        AST::Succ(v) | AST::Pred(v) | AST::IsZero(v) => 1 + arith_depth(v),
        AST::IfThenElse(cond, then, els) => {
            1 + arith_depth(cond)
                .max(arith_depth(then))
                .max(arith_depth(els))
        }
    }
}

#[derive(Debug, Error)]
enum ArithError {
    ParseError(pest::error::Error<Rule>),
    UnexpectedNodeError(Rule),
    UnknownRuleError(AST),
    EmptyPairsError,
}

impl std::fmt::Display for ArithError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)?;
        Ok(())
    }
}

fn try_parse(input: &str) -> Result<AST, ArithError> {
    let input = ArithParser::parse(Rule::Input, input)
        .map_err(|e| ArithError::ParseError(e))?
        .next()
        .ok_or(ArithError::EmptyPairsError)?;
    let input = AST::try_from(input)?;
    Ok(input)
}

fn main() -> Result<(), ArithError> {
    let input = {
        let mut buf = String::new();
        std::io::stdin()
            .read_line(&mut buf)
            .expect("Failed to read input");
        buf.trim_end().to_owned()
    };
    let input = try_parse(input.as_str())?;
    println!("Input: {:?}", input);
    println!(
        "Depth: {}, Size: {}",
        arith_depth(&input),
        arith_size(&input)
    );
    let output = eval_ast(input)?;
    println!("Output: {:?}", output);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() {
        let input = "if iszero pred succ 0 then true else false";
        let input = try_parse(input).unwrap();
        assert_eq!(
            input,
            AST::IfThenElse(
                Box::new(AST::IsZero(Box::new(AST::Pred(Box::new(AST::Succ(
                    Box::new(AST::Zero)
                )))))),
                Box::new(AST::True),
                Box::new(AST::False)
            )
        );
    }

    #[test]
    fn test_numeric_ops() {
        let input = "pred pred succ succ succ 0";
        let input = try_parse(input).unwrap();
        let output = eval_ast(input).unwrap();
        assert_eq!(output, AST::Succ(Box::new(AST::Zero)));
    }

    #[test]
    fn test_eval_if_else() {
        let input = "if iszero succ 0 then true else false";
        let input = try_parse(input).unwrap();
        let output = eval_ast(input).unwrap();
        assert_eq!(output, AST::False);
    }

    #[test]
    fn test_arith_size_and_depth() {
        let input = "if iszero succ 0 then if iszero pred 0 then true else succ 0 else false";
        let input = try_parse(input).unwrap();
        let size = arith_size(&input);
        assert_eq!(size, 12);
        let depth = arith_depth(&input);
        assert_eq!(depth, 5);
    }
}
