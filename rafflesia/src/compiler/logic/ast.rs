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
        statements: InnerStatements,
    },

    IfElseStatement {
        condition: Expression,
        statements: InnerStatements,
        else_statements: InnerStatements,
    },

    RepeatStatement {
        value: Expression,
        statements: InnerStatements,
    },

    ForeverStatement {
        statements: InnerStatements,
    },

    Break,
    Continue,

    // todo: Expression?
}

#[derive(Debug, PartialEq, Clone)]
pub struct Expression {
    // todo
}