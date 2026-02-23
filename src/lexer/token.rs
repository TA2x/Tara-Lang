pub struct Token<T: Debug> {
    pub kind: TokenType, //cant write type as there is already a word 'type' used in rust
    pub lexeme: String, // lexemes like in this exmple 'int x = 5' we have 4 lexemes, 'int' 'x' '='
                        // ' 5'
    pub literal: Option<Literal<T>>,
    pub line: usize, // current line number
}
