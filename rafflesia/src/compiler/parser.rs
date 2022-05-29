use std::fmt::Debug;
use log::{info, trace};
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

    err_tok: T
}

impl<'source, T> LexerWrapper<'source, T>
    where
        T: Logos<'source> + Debug + Clone + PartialEq,
        <<T as Logos<'source>>::Source as Source>::Slice: AsRef<str> {

    pub fn new(inner: Lexer<'source, T>, err_tok: T) -> LexerWrapper<'source, T> {
        LexerWrapper {
            inner,
            cached_tokens: Vec::new(),
            cache_start_point: 0,
            inner_index: 0,
            save_points: vec![0],
            index: 0,
            err_tok
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
        trace!("{}==> New start point", "  ".repeat(self.save_points.len()));
        // push a new state start point
        self.save_points.push(self.index);
    }

    /// Gets the next token, if cached it will use the cache instead.
    ///
    /// Returns an `Err` when either it reaches EOF ([`error::ParseError::EOF`]) or when it
    /// encounters an Error token ([`error::ParseError::LexerError`]).
    pub fn next(&mut self) -> Result<&TokenWrapper<T>, error::ParseError<T, TokenWrapperOwned<T>>> {
        // it's not _really_ necessary to get the current save point,
        // it's only for the sake of consistency
        self.current_save_point()
            .expect("start() must be called first");

        // check if the next token is not already cached
        if self.index >= self.inner_index {
            // yep, this is up-to-date! go next and save it to the cache
            let next_token = self.inner.next()
                .ok_or_else(|| error::ParseError::EOF { expected: None })?;

            // check if this is an error token
            if self.err_tok == next_token {
                // welp return ParseError::LexerError
                return Err(error::ParseError::LexerError {
                    err_token: next_token,
                    pos: self.inner.span(),
                    slice: self.inner.slice().as_ref().to_string(),
                })
            }

            self.cached_tokens.push(TokenWrapper {
                token: next_token,
                slice: self.inner.slice().as_ref(),
                pos: self.inner.span(),
            });

            let ret = self.cached_tokens.get(self.index - self.cache_start_point)
                .ok_or_else(|| error::ParseError::EOF { expected: None })?;

            self.index += 1;
            self.inner_index += 1;

            trace!("{}-> next (now {}, inner {}): {:?}", "  ".repeat(self.save_points.len()), self.index, self.inner_index, ret);
            Ok(ret)
        } else {
            // nope, get it then
            let ret = self.cached_tokens.get(self.index - self.cache_start_point)
                .ok_or_else(|| error::ParseError::EOF { expected: None })?;

            self.index += 1;

            trace!("{}-> next [c] (now {}, inner {}): {:?}", "  ".repeat(self.save_points.len()), self.index, self.inner_index, ret);
            Ok(ret)
        }
    }

    /// Gets the count on how much tokens we've advanced since the last start()
    pub fn get_index(&self) -> usize {
        self.index - self.current_save_point()
            .expect("start() must be called first")
    }

    /// Checks if the next token is as the token given, then return the token; otherwise it will
    /// return a [`error::ParseError::UnexpectedTokenError`]
    pub fn expect(&mut self, tok: T)
        -> Result<TokenWrapperOwned<T>, error::ParseError<T, TokenWrapperOwned<T>>> {
        self.current_save_point()
            .expect("start() must be called first");

        let next: TokenWrapperOwned<T> = self.next()
            .map_err(|err| err.map_eof_expected(|| vec![tok.clone()]))?
            .clone()
            .into();

        if tok == next.token {
            trace!("{} - expected {:?}", "  ".repeat(self.save_points.len()), next);

            Ok(next)
        } else {
            trace!("{} ! unexpected {:?} (expected {:?})", "  ".repeat(self.save_points.len()), next, tok);

            Err(error::ParseError::UnexpectedTokenError {
                expected: Some(vec![tok]),
                pos: next.pos.clone(),
                unexpected_token: next.into(),
            })
        }
    }

    /// Checks if the next token is as the token given, then return the token; otherwise it will
    /// call [`LexerWrapper::previous`] to go back.
    // fixme: LexerError doesnt get propagated
    pub fn expect_failsafe(&mut self, tok: T) -> Option<TokenWrapperOwned<T>> {
        if let Ok(res) = self.expect(tok) { Some(res) } else {
            self.previous();

            None
        }
    }

    /// Expects if the next token is either of the token specified and return the token; otherwise
    /// it will return a [`error::ParseError::UnexpectedTokenError`]
    pub fn expect_multiple_choices(&mut self, tokens: Vec<T>)
        -> Result<TokenWrapperOwned<T>, error::ParseError<T, TokenWrapperOwned<T>>> {
        self.current_save_point()
            .expect("start() must be called first");

        let next: TokenWrapperOwned<T> = self.next()
            .map_err(|err| err.map_eof_expected(|| tokens.clone()))?
            .clone()
            .into();

        if tokens.contains(&next.token) {
            trace!("{} - expected {:?} (expecting mult {:?})", "  ".repeat(self.save_points.len()), next, tokens);

            Ok(next)
        } else {
            trace!("{} ! unexpected {:?} (expecting mult {:?})", "  ".repeat(self.save_points.len()), next, tokens);

            Err(error::ParseError::UnexpectedTokenError {
                expected: Some(tokens),
                pos: next.pos.clone(),
                unexpected_token: next.into(),
            })
        }
    }

    /// Peeks and expects if the next token is either of the token specified and return the token;
    /// otherwise it will return a [`error::ParseError::UnexpectedTokenError`].
    pub fn expect_peek_multiple_choices(&mut self, tokens: Vec<T>)
                                   -> Result<TokenWrapperOwned<T>, error::ParseError<T, TokenWrapperOwned<T>>> {
        trace!("{} - peeking and expecting multiple choices", "  ".repeat(self.save_points.len()));

        let res = self.expect_multiple_choices(tokens);
        self.previous();
        res
    }

    /// Opposite of [`LexerWrapper::expect`]
    pub fn not_expect(&mut self, tok: T)
        -> Result<TokenWrapperOwned<T>, error::ParseError<T, TokenWrapperOwned<T>>> {

        self.current_save_point()
            .expect("start() must be called first");

        let next: TokenWrapperOwned<T> = self.next()?.clone().into();

        if tok != next.token {
            trace!("{} - expected {:?} (not expecting {:?})", "  ".repeat(self.save_points.len()), next, tok);

            Ok(next)
        } else {
            trace!("{} ! unexpected {:?} (not expecting {:?})", "  ".repeat(self.save_points.len()), next, tok);

            Err(error::ParseError::UnexpectedTokenError {
                expected: None,
                pos: next.pos.clone(),
                unexpected_token: next.into(),
            })
        }
    }

    /// Go back one token, will use the cached token.
    ///
    /// Will panic if previous gets called after a [`success()`] (it removes all the cache before
    /// it)
    pub fn previous(&mut self) -> Option<&TokenWrapper<T>> {
        let state_start_point = self.current_save_point()
            .expect("start() must be called first");

        // check if it's trying to get the token before its save point
        if self.index - 1 < *state_start_point || self.index - 1 < self.cache_start_point {
            panic!("trying to access tokens out of bounds");
        }

        // gogogogo
        self.index -= 1;

        let ret = self.cached_tokens.get(self.index - self.cache_start_point);
        trace!("{}<- previous (now {}): {:?}", "  ".repeat(self.save_points.len()), self.index, ret);
        ret
    }

    /// Peeks one token ahead. Basically does [`next()`] and then [`previous()`].
    pub fn peek(&mut self)
        -> Result<TokenWrapperOwned<T>, error::ParseError<T, TokenWrapperOwned<T>>> {

        self.current_save_point()
            .expect("start() must be called first");

        let next = self.next()?.clone().into();
        self.previous();

        trace!("{} - peeked {:?}", "  ".repeat(self.save_points.len()), next);

        Ok(next)
    }

    /// Peeks one token ahead, and expects a token. Then go back no matter what
    pub fn expect_peek(&mut self, tok: T)
        -> Result<TokenWrapperOwned<T>, error::ParseError<T, TokenWrapperOwned<T>>> {

        trace!("{} - expect_peeking {:?}", "  ".repeat(self.save_points.len()), tok);

        let res = self.expect(tok);
        let _ = self.previous().unwrap();

        res
    }

    /// Peeks one token ahead, and not expect a token.
    pub fn not_expect_peek(&mut self, tok: T)
                       -> Result<TokenWrapperOwned<T>, error::ParseError<T, TokenWrapperOwned<T>>> {

        trace!("{} - not_expect_peeking {:?}", "  ".repeat(self.save_points.len()), tok);

        let res = self.not_expect(tok)?;
        let _ = self.previous().unwrap();

        Ok(res)
    }

    /// Restores the LexerWrapper into the previous save point.
    ///
    /// Should be called when a rule parser failed to parse but don't want to propagate the error.
    pub fn restore(&mut self) {
        let state_start_point = self.save_points.pop()
            .expect(
                "Failed to retrieve the previous save point when restoring, is restore() \
                    called after a start()?"
            );

        info!("Restoring lexer to state {}", state_start_point);

        // we just set the index to be the state start point lol
        self.index = state_start_point;
    }

    /// Removes the current save point and **deletes** all the cached tokens before the current index
    pub fn success(&mut self) {
        // prevents splitting when there are multiple success calls at the end of parsing
        // (since there isn't any tokens left)
        if self.index != self.cache_start_point {
            self.cached_tokens =
                self.cached_tokens.split_off(self.index - self.cache_start_point - 1);
        }

        self.cache_start_point = self.index - 1;

        self.save_points.pop()
            .expect("Failed to pop the previous save point, is success() called after a start()?");

        trace!("{}<== success ({})", "  ".repeat(self.save_points.len() + 1), self.index);
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
            pos: std::ops::Range<usize>,
        },

        // end of file
        EOF {
            expected: Option<Vec<ET>>
        },

        // when the lexer wrapper encounters an Error token
        LexerError {
            err_token: ET,
            pos: std::ops::Range<usize>,
            slice: String
        }
    }

    /// Propagates non recoverable errors (LexerError, and EOF)
    macro_rules! propagate_non_recoverable {
        ($rule: expr) => {
            match $rule {
                Ok(res) => Ok(res),
                Err(er) => if !er.is_recoverable() { return Err(er) } else { Err(er) }
            }
        };
    }

    /// Propagates non recoverable errors except for EOF (only propagates LexerError)
    macro_rules! propagate_non_recoverable_wo_eof {
        ($rule: expr) => {
            match $rule {
                Ok(res) => Ok(res),
                Err(er) => if !er.is_recoverable_w_eof() { return Err(er) } else { Err(er) }
            }
        };
    }

    pub(crate) use propagate_non_recoverable;
    pub(crate) use propagate_non_recoverable_wo_eof;

    impl<ET: Debug, UET: Debug> ParseError<ET, UET> {
        /// Returns whether the error is recoverable (an unexpected token), or is unrecoverable
        /// (EOF, Error token)
        pub fn is_recoverable(&self) -> bool {
            match self {
                ParseError::UnexpectedTokenError { .. } => true,
                ParseError::EOF { .. } => false,
                ParseError::LexerError { .. } => false,
            }
        }

        /// Does the same thing as [`ParseError::is_recoverable`], except that EOF returns true.
        /// Used in situations where EOF might be a sign of an uncomplete rule at the end of the
        /// file.
        pub fn is_recoverable_w_eof(&self) -> bool {
            match self {
                ParseError::UnexpectedTokenError { .. } => true,
                ParseError::EOF { .. } => true,
                ParseError::LexerError { .. } => false,
            }
        }
        
        /// Maps the `expected` field of [`ParseError::EOF`]
        pub fn map_eof_expected<F>(self, f: F) -> Self
        where F: FnOnce() -> Vec<ET> {
            match self {
                ParseError::EOF { .. } => ParseError::EOF { expected: Some(f()) },
                _ => self
            }
        }
    }

    impl<ET: Debug, UET: Debug> Debug for ParseError<ET, UET> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                ParseError::UnexpectedTokenError { expected, unexpected_token, .. } => {
                    if let Some(e) = expected {
                        if e.len() == 1 {
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
                ParseError::EOF { expected } => {
                    if let Some(expected) = expected {
                        if expected.len() == 1 {
                            write!(
                                f,
                                "expected token {:?}, but reached end-of-file",
                                expected.get(0)
                            )
                        } else {
                            write!(
                                f,
                                "expected a {}, but reached end-of-file",
                                expected.iter()
                                    .fold(String::new(), |acc, tok| {
                                        format!("{:?} or", tok)
                                    })[..3].to_string(), // removes the trailing ` or`
                            )
                        }
                    } else {
                        write!(f, "reached end-of-file")
                    }
                }
                ParseError::LexerError { err_token, pos, slice } => {
                    write!(f, "lexer error {:?} at {:?}: `{}`", err_token, pos, slice)
                }
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