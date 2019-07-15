use crate::ast::*;
use crate::parser::*;
use nom::branch::*;
use nom::combinator::*;
use nom::multi::*;
use nom::sequence::*;
use nom::IResult;

// -----------------------------------------------------------------------------

#[derive(Debug, Node)]
pub struct ConstraintDeclaration<'a> {
    pub nodes: (
        Option<Static<'a>>,
        Symbol<'a>,
        ConstraintIdentifier<'a>,
        ConstraintBlock<'a>,
    ),
}

#[derive(Debug, Node)]
pub struct Static<'a> {
    pub nodes: (Symbol<'a>,),
}

#[derive(Debug, Node)]
pub struct ConstraintBlock<'a> {
    pub nodes: (Brace<'a, Vec<ConstraintBlockItem<'a>>>,),
}

#[derive(Debug, Node)]
pub enum ConstraintBlockItem<'a> {
    Solve(ConstraintBlockItemSolve<'a>),
    ConstraintExpression(ConstraintExpression<'a>),
}

#[derive(Debug, Node)]
pub struct ConstraintBlockItemSolve<'a> {
    pub nodes: (
        Symbol<'a>,
        SolveBeforeList<'a>,
        Symbol<'a>,
        SolveBeforeList<'a>,
        Symbol<'a>,
    ),
}

#[derive(Debug, Node)]
pub struct SolveBeforeList<'a> {
    pub nodes: (List<Symbol<'a>, ConstraintPrimary<'a>>,),
}

#[derive(Debug, Node)]
pub struct ConstraintPrimary<'a> {
    pub nodes: (
        Option<ImplicitClassHandleOrClassScope<'a>>,
        HierarchicalIdentifier<'a>,
        Select<'a>,
    ),
}

#[derive(Debug, Node)]
pub enum ConstraintExpression<'a> {
    Expression(ConstraintExpressionExpression<'a>),
    UniquenessConstraint((UniquenessConstraint<'a>, Symbol<'a>)),
    Arrow(ConstraintExpressionArrow<'a>),
    If(ConstraintExpressionIf<'a>),
    Foreach(ConstraintExpressionForeach<'a>),
    Disable(ConstraintExpressionDisable<'a>),
}

#[derive(Debug, Node)]
pub struct ConstraintExpressionExpression<'a> {
    pub nodes: (Option<Soft<'a>>, ExpressionOrDist<'a>, Symbol<'a>),
}

#[derive(Debug, Node)]
pub struct Soft<'a> {
    pub nodes: (Symbol<'a>,),
}

#[derive(Debug, Node)]
pub struct ConstraintExpressionArrow<'a> {
    pub nodes: (Expression<'a>, Symbol<'a>, ConstraintSet<'a>),
}

#[derive(Debug, Node)]
pub struct ConstraintExpressionIf<'a> {
    pub nodes: (
        Symbol<'a>,
        Paren<'a, Expression<'a>>,
        ConstraintSet<'a>,
        Option<(Symbol<'a>, ConstraintSet<'a>)>,
    ),
}

#[derive(Debug, Node)]
pub struct ConstraintExpressionForeach<'a> {
    pub nodes: (
        Symbol<'a>,
        Paren<
            'a,
            (
                PsOrHierarchicalArrayIdentifier<'a>,
                Bracket<'a, LoopVariables<'a>>,
            ),
        >,
        ConstraintSet<'a>,
    ),
}

#[derive(Debug, Node)]
pub struct ConstraintExpressionDisable<'a> {
    pub nodes: (Symbol<'a>, Symbol<'a>, ConstraintPrimary<'a>, Symbol<'a>),
}

#[derive(Debug, Node)]
pub struct UniquenessConstraint<'a> {
    pub nodes: (Symbol<'a>, Brace<'a, OpenRangeList<'a>>),
}

#[derive(Debug, Node)]
pub enum ConstraintSet<'a> {
    ConstraintExpression(Box<ConstraintExpression<'a>>),
    Brace(ConstraintSetBrace<'a>),
}

#[derive(Debug, Node)]
pub struct ConstraintSetBrace<'a> {
    pub nodes: (Brace<'a, Vec<ConstraintExpression<'a>>>,),
}

#[derive(Debug, Node)]
pub struct DistList<'a> {
    pub nodes: (List<Symbol<'a>, DistItem<'a>>,),
}

#[derive(Debug, Node)]
pub struct DistItem<'a> {
    pub nodes: (ValueRange<'a>, Option<DistWeight<'a>>),
}

#[derive(Debug, Node)]
pub enum DistWeight<'a> {
    Equal(DistWeightEqual<'a>),
    Divide(DistWeightDivide<'a>),
}

#[derive(Debug, Node)]
pub struct DistWeightEqual<'a> {
    pub nodes: (Symbol<'a>, Expression<'a>),
}

#[derive(Debug, Node)]
pub struct DistWeightDivide<'a> {
    pub nodes: (Symbol<'a>, Expression<'a>),
}

#[derive(Debug, Node)]
pub struct ConstraintPrototype<'a> {
    pub nodes: (
        Option<ConstraintPrototypeQualifier<'a>>,
        Option<Static<'a>>,
        Symbol<'a>,
        ConstraintIdentifier<'a>,
        Symbol<'a>,
    ),
}

#[derive(Debug, Node)]
pub enum ConstraintPrototypeQualifier<'a> {
    Extern(Symbol<'a>),
    Pure(Symbol<'a>),
}

#[derive(Debug, Node)]
pub struct ExternConstraintDeclaration<'a> {
    pub nodes: (
        Option<Static<'a>>,
        Symbol<'a>,
        ClassScope<'a>,
        ConstraintIdentifier<'a>,
        ConstraintBlock<'a>,
    ),
}

#[derive(Debug, Node)]
pub struct IdentifierList<'a> {
    pub nodes: (List<Symbol<'a>, Identifier<'a>>,),
}

// -----------------------------------------------------------------------------

pub fn constraint_declaration(s: Span) -> IResult<Span, ConstraintDeclaration> {
    let (s, a) = opt(r#static)(s)?;
    let (s, b) = symbol("constraint")(s)?;
    let (s, c) = constraint_identifier(s)?;
    let (s, d) = constraint_block(s)?;
    Ok((
        s,
        ConstraintDeclaration {
            nodes: (a, b, c, d),
        },
    ))
}

pub fn r#static(s: Span) -> IResult<Span, Static> {
    let (s, a) = symbol("static")(s)?;
    Ok((s, Static { nodes: (a,) }))
}

pub fn constraint_block(s: Span) -> IResult<Span, ConstraintBlock> {
    let (s, a) = brace(many0(constraint_block_item))(s)?;
    Ok((s, ConstraintBlock { nodes: (a,) }))
}

pub fn constraint_block_item(s: Span) -> IResult<Span, ConstraintBlockItem> {
    alt((
        constraint_block_item_solve,
        map(constraint_expression, |x| {
            ConstraintBlockItem::ConstraintExpression(x)
        }),
    ))(s)
}

pub fn constraint_block_item_solve(s: Span) -> IResult<Span, ConstraintBlockItem> {
    let (s, a) = symbol("solve")(s)?;
    let (s, b) = solve_before_list(s)?;
    let (s, c) = symbol("before")(s)?;
    let (s, d) = solve_before_list(s)?;
    let (s, e) = symbol(";")(s)?;
    Ok((
        s,
        ConstraintBlockItem::Solve(ConstraintBlockItemSolve {
            nodes: (a, b, c, d, e),
        }),
    ))
}

pub fn solve_before_list(s: Span) -> IResult<Span, SolveBeforeList> {
    let (s, a) = list(symbol(","), constraint_primary)(s)?;
    Ok((s, SolveBeforeList { nodes: (a,) }))
}

pub fn constraint_primary(s: Span) -> IResult<Span, ConstraintPrimary> {
    let (s, a) = opt(implicit_class_handle_or_class_scope)(s)?;
    let (s, b) = hierarchical_identifier(s)?;
    let (s, c) = select(s)?;
    Ok((s, ConstraintPrimary { nodes: (a, b, c) }))
}

pub fn constraint_expression(s: Span) -> IResult<Span, ConstraintExpression> {
    alt((
        constraint_expression_expression,
        map(pair(uniqueness_constraint, symbol(";")), |x| {
            ConstraintExpression::UniquenessConstraint(x)
        }),
        constraint_expression_arrow,
        constraint_expression_if,
        constraint_expression_foreach,
        constraint_expression_disable,
    ))(s)
}

pub fn constraint_expression_expression(s: Span) -> IResult<Span, ConstraintExpression> {
    let (s, a) = opt(soft)(s)?;
    let (s, b) = expression_or_dist(s)?;
    let (s, c) = symbol(";")(s)?;
    Ok((
        s,
        ConstraintExpression::Expression(ConstraintExpressionExpression { nodes: (a, b, c) }),
    ))
}

pub fn soft(s: Span) -> IResult<Span, Soft> {
    let (s, a) = symbol("soft")(s)?;
    Ok((s, Soft { nodes: (a,) }))
}

pub fn constraint_expression_arrow(s: Span) -> IResult<Span, ConstraintExpression> {
    let (s, a) = expression(s)?;
    let (s, b) = symbol("->")(s)?;
    let (s, c) = constraint_set(s)?;
    Ok((
        s,
        ConstraintExpression::Arrow(ConstraintExpressionArrow { nodes: (a, b, c) }),
    ))
}

pub fn constraint_expression_if(s: Span) -> IResult<Span, ConstraintExpression> {
    let (s, a) = symbol("if")(s)?;
    let (s, b) = paren(expression)(s)?;
    let (s, c) = constraint_set(s)?;
    let (s, d) = opt(pair(symbol("else"), constraint_set))(s)?;
    Ok((
        s,
        ConstraintExpression::If(ConstraintExpressionIf {
            nodes: (a, b, c, d),
        }),
    ))
}

pub fn constraint_expression_foreach(s: Span) -> IResult<Span, ConstraintExpression> {
    let (s, a) = symbol("foreach")(s)?;
    let (s, b) = paren(pair(
        ps_or_hierarchical_array_identifier,
        bracket(loop_variables),
    ))(s)?;
    let (s, c) = constraint_set(s)?;
    Ok((
        s,
        ConstraintExpression::Foreach(ConstraintExpressionForeach { nodes: (a, b, c) }),
    ))
}

pub fn constraint_expression_disable(s: Span) -> IResult<Span, ConstraintExpression> {
    let (s, a) = symbol("disable")(s)?;
    let (s, b) = symbol("soft")(s)?;
    let (s, c) = constraint_primary(s)?;
    let (s, d) = symbol(";")(s)?;
    Ok((
        s,
        ConstraintExpression::Disable(ConstraintExpressionDisable {
            nodes: (a, b, c, d),
        }),
    ))
}

pub fn uniqueness_constraint(s: Span) -> IResult<Span, UniquenessConstraint> {
    let (s, a) = symbol("unique")(s)?;
    let (s, b) = brace(open_range_list)(s)?;
    Ok((s, UniquenessConstraint { nodes: (a, b) }))
}

pub fn constraint_set(s: Span) -> IResult<Span, ConstraintSet> {
    alt((
        map(constraint_expression, |x| {
            ConstraintSet::ConstraintExpression(Box::new(x))
        }),
        constraint_set_brace,
    ))(s)
}

pub fn constraint_set_brace(s: Span) -> IResult<Span, ConstraintSet> {
    let (s, a) = brace(many0(constraint_expression))(s)?;
    Ok((s, ConstraintSet::Brace(ConstraintSetBrace { nodes: (a,) })))
}

pub fn dist_list(s: Span) -> IResult<Span, DistList> {
    let (s, a) = list(symbol(","), dist_item)(s)?;
    Ok((s, DistList { nodes: (a,) }))
}

pub fn dist_item(s: Span) -> IResult<Span, DistItem> {
    let (s, a) = value_range(s)?;
    let (s, b) = opt(dist_weight)(s)?;
    Ok((s, DistItem { nodes: (a, b) }))
}

pub fn dist_weight(s: Span) -> IResult<Span, DistWeight> {
    alt((dist_weight_equal, dist_weight_divide))(s)
}

pub fn dist_weight_equal(s: Span) -> IResult<Span, DistWeight> {
    let (s, a) = symbol(":=")(s)?;
    let (s, b) = expression(s)?;
    Ok((s, DistWeight::Equal(DistWeightEqual { nodes: (a, b) })))
}

pub fn dist_weight_divide(s: Span) -> IResult<Span, DistWeight> {
    let (s, a) = symbol(":/")(s)?;
    let (s, b) = expression(s)?;
    Ok((s, DistWeight::Divide(DistWeightDivide { nodes: (a, b) })))
}

pub fn constraint_prototype(s: Span) -> IResult<Span, ConstraintPrototype> {
    let (s, a) = opt(constraint_prototype_qualifier)(s)?;
    let (s, b) = opt(r#static)(s)?;
    let (s, c) = symbol("constraint")(s)?;
    let (s, d) = constraint_identifier(s)?;
    let (s, e) = symbol(";")(s)?;
    Ok((
        s,
        ConstraintPrototype {
            nodes: (a, b, c, d, e),
        },
    ))
}

pub fn constraint_prototype_qualifier(s: Span) -> IResult<Span, ConstraintPrototypeQualifier> {
    alt((
        map(symbol("extern"), |x| {
            ConstraintPrototypeQualifier::Extern(x)
        }),
        map(symbol("pure"), |x| ConstraintPrototypeQualifier::Pure(x)),
    ))(s)
}

pub fn extern_constraint_declaration(s: Span) -> IResult<Span, ExternConstraintDeclaration> {
    let (s, a) = opt(r#static)(s)?;
    let (s, b) = symbol("constraint")(s)?;
    let (s, c) = class_scope(s)?;
    let (s, d) = constraint_identifier(s)?;
    let (s, e) = constraint_block(s)?;
    Ok((
        s,
        ExternConstraintDeclaration {
            nodes: (a, b, c, d, e),
        },
    ))
}

pub fn identifier_list(s: Span) -> IResult<Span, IdentifierList> {
    let (s, a) = list(symbol(","), identifier)(s)?;
    Ok((s, IdentifierList { nodes: (a,) }))
}
