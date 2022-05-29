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
        // for later
        // initial_value: Option<Expression>,
        identifier: String,
    },

    ComplexVariableDeclaration {
        variable_type: ComplexVariableType,
        identifier: String,
    },

    ActivityEventListener {
        event_name: String,
        body: InnerStatements
    },

    ViewEventListener {
        view_id: String,
        event_name: String,
        body: InnerStatements
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct InnerStatements(pub Vec<InnerStatement>);

#[derive(Debug, PartialEq, Clone)]
pub struct VariableAssignment {
    pub identifier: String,
    pub value: Expression
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfStatement {
    pub condition: Expression,
    pub body: InnerStatements,
    pub else_body: Option<InnerStatements>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ForeverStatement {
    pub body: InnerStatements,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RepeatStatement {
    pub condition: Expression,
    pub body: InnerStatements,
}

#[derive(Debug, PartialEq, Clone)]
pub enum InnerStatement {
    VariableAssignment(VariableAssignment),
    IfStatement(IfStatement),
    RepeatStatement(RepeatStatement),
    ForeverStatement(ForeverStatement),
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
        from: Box<Expression>, // using PrimaryExpression would be better
        index: Box<Expression>,
    },
    VariableAccess {
        // from.name if Some
        // fixme: there's a better approach to this
        from: Option<Box<Expression>>, // using PrimaryExpression would be better
        name: String,
    },
    // calling a VariableAccess with arguments
    Call {
        // from(arguments)
        from: Box<Expression>, // using PrimaryExpression would be better
        arguments: Arguments
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Arguments(pub Vec<Expression>);