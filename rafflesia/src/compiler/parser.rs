use std::fmt::Debug;
use logos::{Lexer, Logos, Source};

#[derive(Debug, Clone, PartialEq)]
pub struct TokenWrapper<'source, T: Debug + Clone + PartialEq> {
    pub token: T,
    pub slice: &'source str,
    pub pos: std::ops::Range<usize>
}

// i think there's a better way of doing this
#[derive(Debug, Clone, PartialEq)]
pub struct TokenWrapperOwned<T: Debug + Clone + PartialEq> {
    pub token: T,
    pub slice: String,
    pub pos: std::ops::Range<usize>
}

impl<T: Debug + Clone + PartialEq> From<TokenWrapper<'_, T>> for TokenWrapperOwned<T> {
    fn from(tok: TokenWrapper<T>) -> Self {
        TokenWrapperOwned {
            token: tok.token,
            slice: tok.slice.to_string(),
            pos: tok.pos
        }
    }
}

pub struct LexerWrapper<'source, T: Logos<'source> + Debug + Clone + PartialEq>
    where
    // This code makes me want to scream. What this does is to constrain the slice of the source
    // to only be a &str, as it can be a &[u8] depending on the lexer.
    // read logos' source at 1ecc6299db9ec823 or 0.12.0 at /src/lib.rs:190

    // A better idea is to probably create another type parameter for the slice, but that'll be
    // a bit too generic for my usage. If you do need it, you should change it yourself.
        <<T as Logos<'source>>::Source as Source>::Slice: AsRef<str> {

    inner: Lexer<'source, T>,
    cached_tokens: Vec<TokenWrapper<'source, T>>,
    cache_start_point: usize,
    inner_index: usize,

    save_points: Vec<usize>,
    index: usize,
}

impl<'source, T> LexerWrapper<'source, T>
    where
        T: Logos<'source> + Debug + Clone + PartialEq,
        <<T as Logos<'source>>::Source as Source>::Slice: AsRef<str> {

    pub fn new(inner: Lexer<'source, T>) -> LexerWrapper<'source, T> {
        LexerWrapper {
            inner,
            cached_tokens: Vec::new(),
            cache_start_point: 0,
            inner_index: 0,
            save_points: vec![0],
            index: 0,
        }
    }

    #[inline]
    fn current_save_point(&self) -> Option<&usize> {
        self.save_points.get(self.save_points.len() - 1)
    }

    /// Called at the start of the parsing of a grammar; it will create a new save point or
    /// "checkpoint" in which the Lexer can be restored onto by calling [`restore()`], unless it
    /// got removed by calling [`success()`].
    pub fn start(&mut self) {
        // push a new state start point
        self.save_points.push(self.index);
    }

    pub fn next(&mut self) -> Option<&TokenWrapper<T>> {
        // it's not _really_ necessary to get the current save point,
        // it's only for the sake of consistency
        self.current_save_point()
            .expect("start() must be called first");

        // check if the next token is in the cache
        if self.index > self.inner_index {
            // yes, get it then
            self.index += 1;

            self.cached_tokens.get(self.index - self.cache_start_point)
        } else {
            // nope, this is up-to-date! go next and save it to the cache
            let next_token = self.inner.next()?; // will return None if there is none left

            self.cached_tokens.push(TokenWrapper {
                token: next_token,
                slice: self.inner.slice().as_ref(),
                pos: self.inner.span(),
            });

            self.index += 1;
            self.inner_index += 1;

            Some(&self.cached_tokens.get(self.index - self.cache_start_point).unwrap())
        }
    }

    /// Retrieves on how much tokens we've advanced since the last start()
    pub fn get_index(&self) -> usize {
        self.index - self.current_save_point()
            .expect("start() must be called first")
    }

    /// Checks if the next token is as the token given, then return the token; otherwise it will
    /// return a [`error::ParseError::UnexpectedTokenError`].
    pub fn expect(&mut self, tok: T)
        -> Result<TokenWrapperOwned<T>, error::ParseError<T, TokenWrapperOwned<T>>> {

        let next: TokenWrapperOwned<T> = self.next().ok_or_else(||error::ParseError::EOF)?.clone().into();

        if tok == next.token {
            Ok(next)
        } else {
            self.restore();

            Err(error::ParseError::UnexpectedTokenError {
                expected: Some(tok),
                range: next.pos.clone(),
                unexpected_token: next.into(),
            })
        }
    }

    /// Opposite of [`LexerWrapper::expect`]
    pub fn not_expect(&mut self, tok: T)
        -> Result<TokenWrapperOwned<T>, error::ParseError<T, TokenWrapperOwned<T>>> {

        let next: TokenWrapperOwned<T> = self.next().ok_or_else(||error::ParseError::EOF)?.clone().into();

        if tok != next.token {
            Ok(next)
        } else {
            self.restore();

            Err(error::ParseError::UnexpectedTokenError {
                expected: None,
                range: next.pos.clone(),
                unexpected_token: next.into(),
            })
        }
    }

    pub fn previous(&mut self) -> Option<&T> {
        let state_start_point = self.current_save_point()
            .expect("start() must be called first");

        // check if it's trying to get the token before its save point
        if self.index - 1 < *state_start_point {
            panic!("trying to access tokens out of bounds");
        }

        // gogogogo
        self.index -= 1;

        self.cached_tokens.get(self.index - self.cache_start_point)
            .map(|c| &c.token)
    }

    /// Restores the LexerWrapper into the previous save point
    pub fn restore(&mut self) {
        let state_start_point = self.save_points.pop()
            .expect(
                "Failed to retrieve the previous save point when restoring, is restore() \
                    called after a start()?"
            );

        // we just set the index to be the state start point lol
        self.index = state_start_point;
    }

    /// Removes the current save point and deletes all the cached tokens before the current index
    pub fn success(&mut self) {
        self.cached_tokens =
            self.cached_tokens.split_off(self.index - self.cache_start_point - 1);

        self.save_points.pop()
            .expect("Failed to pop the previous save point, is success() called after a start()?");
    }
}

pub mod error {
    use std::error::Error;
    use std::fmt::{Debug, Display, Formatter};

    pub enum ParseError<ET: Debug, UET: Debug> {
        // unexpected token
        UnexpectedTokenError {
            expected: Option<Vec<ET>>,
            unexpected_token: UET,
            range: std::ops::Range<usize>,
        },

        // end of file
        EOF
    }

    impl<ET: Debug, UET: Debug> Debug for ParseError<ET, UET> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                ParseError::UnexpectedTokenError { expected, unexpected_token, .. } => {
                    if let Some(e) = expected {
                        if e.len() == 0 {
                            write!(
                                f,
                                "expected token {:?}, got {:?} instead", e.get(0), unexpected_token
                            )
                        } else {
                            write!(
                                f,
                                "expected a {}, got {:?} instead",
                                e.iter()
                                    .fold(String::new(), |acc, tok| {
                                        format!("{:?} or", tok)
                                    })[..3].to_string(), // removes the trailing ` or`
                                unexpected_token
                            )
                        }
                    } else {
                        write!(f, "unexpected token {:?}", unexpected_token)
                    }
                },
                ParseError::EOF => write!(f, "reached end-of-file")
            }
        }
    }

    impl<ET: Debug, UET: Debug> Display for ParseError<ET, UET> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            <Self as Debug>::fmt(self, f)
        }
    }

    impl<ET: Debug, UET: Debug> Error for ParseError<ET, UET> {}
}