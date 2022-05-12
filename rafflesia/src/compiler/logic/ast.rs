#[derive(Debug, PartialEq, Clone)]
pub enum VariableType {
    Number,
    String,
    Boolean,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ComplexVariableType {
    Map { inner_type: VariableType },
    List { inner_type: VariableType },
}

#[derive(Debug, PartialEq, Clone)]
pub struct OuterStatements(pub Vec<OuterStatement>);

#[derive(Debug, PartialEq, Clone)]
pub enum OuterStatement {
    SimpleVariableDeclaration {
        variable_type: VariableType,
        initial_value: Option<Expression>,
        identifier: String,
    },

    ComplexVariableDeclaration {
        variable_type: ComplexVariableType,
        identifier: String,
    },

    ActivityEventListener {
        event_name: String,
        statements: InnerStatements
    },

    ViewEventListener {
        view_id: String,
        event_name: String,
        statements: InnerStatements
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct InnerStatements(pub Vec<InnerStatement>);

#[derive(Debug, PartialEq, Clone)]
pub enum InnerStatement {
    VariableAssignment {
        identifier: String,
        value: Expression,
    },

    IfStatement {
        condition: Expression,
        body: InnerStatements,
    },

    IfElseStatement {
        condition: Expression,
        body: InnerStatements,
        else_body: InnerStatements,
    },

    RepeatStatement {
        value: Expression,
        body: InnerStatements,
    },

    ForeverStatement {
        body: InnerStatements,
    },

    Break,
    Continue,
    Expression(Expression)
}

#[derive(Debug, PartialEq, Clone)]
pub enum BinaryOperator {
    Or,
    And,
    LT,
    LTE,
    GT,
    GTE,
    EQ,
    Plus,
    Minus,
    Multiply,
    Divide,
    Power,
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOperator {
    Not,
    Minus,
    Plus,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    BinOp {
        first: Box<Expression>,
        operator: BinaryOperator,
        second: Box<Expression>,
    },
    UnaryOp {
        value: Box<Expression>,
        operator: UnaryOperator
    },
    PrimaryExpression(PrimaryExpression),
    Literal(Literal)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Number(f64),
    Boolean(bool),
    String(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum PrimaryExpression {
    // from[index]
    Index {
        from: Box<PrimaryExpression>,
        index: Box<Expression>,
    },
    VariableAccess {
        // from.name if Some
        // fixme: there's a better approach to this
        from: Option<Box<PrimaryExpression>>,
        name: String,
    },
    // name(arguments)
    FunctionCall {
        // from.name(arguments) if Some
        // fixme: there's a better approach to this
        from: Option<Box<PrimaryExpression>>,
        name: String,
        arguments: Arguments
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Arguments(pub Vec<Expression>);