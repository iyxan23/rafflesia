//! # Buffered Lexer
//! A wrapper to logos' [`logos::Lexer`] that adds a functionality where Lexer is able to be
//! saved at a point and restored later on, with some caching/buffering functionalities and
//! advanced token-advancing functions like [`BufferedLexer::expect`],
//! [`BufferedLexer::expect_failsafe`], etc. that allows you to surf through tokens with ease.
//! 
//! You can construct an instance of this Lexer by passing the already provided logos'
//! [`logos::Lexer`] and the error token of your logos-generated token enum.
//! 
//! ```
//! use logos::Logos;
//! use buffered_lexer::BufferedLexer;
//! 
//! // Your logos-generated Token
//! #[derive(Logos, Debug, Clone, PartialEq)]
//! enum Token {
//!     // ...
//! 
//!     #[error]
//!     Error
//! }
//!
//! let raw = "Hello world";
//! let mut lexer: BufferedLexer<'_, Token> = BufferedLexer::new(Token::lexer(raw), Token::Error);
//! // start using `lexer`
//! ```
//! 
//! To parse with this lexer, you must first understand the concept of "check points".
//! 
//! Checkpoints in [`BufferedLexer`] is basically a stack that stores previous points
//! where it got saved. You may create a new saving point using [`BufferedLexer::start`],
//! close a saving point with [`BufferedLexer::success`], and restore to the previous
//! point using [`BufferedLexer::restore`].
//! 
//! Using this parser is like any parsers out there. [`BufferedLexer`] provides the function
//! [`BufferedLexer::next`] of which to advance the parser's cursor a token forward. But 
//! this parser also provides extra functionalities that are:
//! 
//!  - **[`BufferedLexer::expect`]**
//! 
//!    Checks if the next token is as the token given, then return the token as
//!    [`SpannedTokenOwned`]. If otherwise, return a [`error::ParseError::UnexpectedTokenError`].
//! 
//!  - **[`BufferedLexer::expect_failsafe`]**
//! 
//!    Checks if the next token is as the token given, then return the token. Otherwise, go back
//!    when the error is unexpected token. Will not go back when the errors are either LexerError
//!    or EOF.
//! 
//!  - **[`BufferedLexer::expect_multiple_choices`]**
//! 
//!    Expects if the next token is either of the token specified, then return the token.
//!    Otherwise, return a [`error::ParseError::UnexpectedTokenError`].
//! 
//!  - todo

use std::fmt::Debug;
use log::{info, trace};
use logos::{Lexer, Logos, Source};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken<'source, T: Debug + Clone + PartialEq> {
    pub token: T,
    pub slice: &'source str,
    pub pos: std::ops::Range<usize>
}

// i think there's a better way of doing this
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedTokenOwned<T: Debug + Clone + PartialEq> {
    pub token: T,
    pub slice: String,
    pub pos: std::ops::Range<usize>
}

impl<T: Debug + Clone + PartialEq> From<SpannedToken<'_, T>> for SpannedTokenOwned<T> {
    fn from(tok: SpannedToken<T>) -> Self {
        SpannedTokenOwned {
            token: tok.token,
            slice: tok.slice.to_string(),
            pos: tok.pos
        }
    }
}

/// [`BufferedLexer`] is a wrapper to the Logos' [`logos::Lexer`] that implements token buffering, saving contexts and restoring them.
pub struct BufferedLexer<'source, T: Logos<'source> + Debug + Clone + PartialEq>
    where
    // This code makes me want to scream. What this does is to constrain the slice of the source
    // to only be a &str, as it can be a &[u8] depending on the lexer.
    // read logos' source at 1ecc6299db9ec823 or 0.12.0 at /src/lib.rs:190

    // A better idea is to probably create another type parameter for the slice, but that'll be
    // a bit too generic for my usage. If you do need it, you should change it yourself.
        <<T as Logos<'source>>::Source as Source>::Slice: AsRef<str> {

    inner: Lexer<'source, T>,
    cached_tokens: Vec<SpannedToken<'source, T>>,
    cache_start_point: usize,
    inner_index: usize,

    save_points: Vec<usize>,
    blacklist: Vec<T>,
    index: usize,

    err_tok: T
}

impl<'source, T> BufferedLexer<'source, T>
    where
        T: Logos<'source> + Debug + Clone + PartialEq,
        <<T as Logos<'source>>::Source as Source>::Slice: AsRef<str> {

    /// Constructs a new [`BufferedLexer`] instance from a logos' generated Lexer ([`logos::Lexer`]).
    /// This function takes an error token so in a case where the logos lexer encounters
    /// an error while lexing, we can transform it into our own error [`error::ParseError::LexerError`].
    ///
    /// Calling this function will automatically start a new starting point. (As if you called
    /// [`BufferedLexer::start`] after this new function).
    pub fn new(inner: Lexer<'source, T>, err_tok: T) -> BufferedLexer<'source, T> {
        BufferedLexer {
            inner,
            cached_tokens: Vec::new(),
            cache_start_point: 0,
            inner_index: 0,
            save_points: vec![0],
            blacklist: vec![],
            index: 0,
            err_tok
        }
    }

    #[inline]
    fn current_save_point(&self) -> Option<&usize> {
        self.save_points.get(self.save_points.len() - 1)
    }

    /// Called at the start of the parsing of a grammar; it will create a new save point or
    /// "checkpoint" in which the Lexer can be restored onto by calling [`BufferedLexer::restore`]
    /// , unless it got removed by calling [`BufferedLexer::success`].
    pub fn start(&mut self) {
        trace!("{}==> New start point", "  ".repeat(self.save_points.len()));
        // push a new state start point
        self.save_points.push(self.index);
    }

    /// Gets the next token, if cached it will use the cache instead.
    ///
    /// Returns an `Err` when either it reaches EOF ([`error::ParseError::EOF`]) or when it
    /// encounters an Error token ([`error::ParseError::LexerError`]).
    pub fn next(&mut self) -> Result<&SpannedToken<T>, error::ParseError<T, SpannedTokenOwned<T>>> {
        // it's not _really_ necessary to get the current save point,
        // it's only for the sake of consistency
        self.current_save_point()
            .expect("start() must be called first");

        // check if the next token is not already cached
        if self.index >= self.inner_index {
            // yep, this is up-to-date! go next and save it to the cache
            let next_token = self.inner.next()
                .ok_or_else(|| {
                    trace!(
                        "{} * encountered eof (now {}, inner {})",
                        "  ".repeat(self.save_points.len()), self.index, self.inner_index
                    );
                    error::ParseError::EOF { expected: None }
                })?;

            // check if this is an error token
            if self.err_tok == next_token {
                // welp return ParseError::LexerError
                return Err(error::ParseError::LexerError {
                    err_token: next_token,
                    pos: self.inner.span(),
                    slice: self.inner.slice().as_ref().to_string(),
                })
            }

            self.cached_tokens.push(SpannedToken {
                token: next_token,
                slice: self.inner.slice().as_ref(),
                pos: self.inner.span(),
            });

            let ret = self.cached_tokens.get(self.index - self.cache_start_point)
                .ok_or_else(|| {
                    trace!(
                        "{} * encountered eof (now {}, inner {})",
                        "  ".repeat(self.save_points.len()), self.index, self.inner_index
                    );
                    error::ParseError::EOF { expected: None }
                })?;

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
        -> Result<SpannedTokenOwned<T>, error::ParseError<T, SpannedTokenOwned<T>>> {
        self.current_save_point()
            .expect("start() must be called first");

        // loop until the next token is not blacklisted
        let next = loop {
            let next: SpannedTokenOwned<T> = self.next()
                .map_err(|err| err.map_eof_expected(|| vec![tok.clone()]))?
                .clone()
                .into();

            if !self.blacklist.contains(&next.token) { break next; }
        };


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

    /// Checks if the next token is as the token given, then return the token. Otherwise, go back
    /// when the error is unexpected token. Will not go back when the errors are either LexerError
    /// or EOF
    pub fn expect_failsafe(&mut self, tok: T)
        -> Result<Option<SpannedTokenOwned<T>>, error::ParseError<T, SpannedTokenOwned<T>>> {
        trace!("{} - expecting [failsafe] {:?}", "  ".repeat(self.save_points.len()), tok);

        match self.expect(tok) {
            Ok(res) => {
                trace!("{} - expected [failsafe] {:?}", "  ".repeat(self.save_points.len()), &res);

                Ok(Some(res))
            }

            Err(err) => {
                // only go back when the error is recoverable (unexpected token)
                // we cannot go back on errors like EOF and LexerError
                if err.is_recoverable() {
                    trace!("{} v unexpected [failsafe]", "  ".repeat(self.save_points.len()));
                    self.previous();
                    Ok(None)
                } else {
                    trace!(
                        "{} v unexpected [failsafe] irrecoverable err: {}",
                        "  ".repeat(self.save_points.len()), err
                    );
                    Err(err)
                }
            }
        }
    }

    /// Checks if the next token is as the token given, then return the token. Otherwise, go back
    /// when the error is unexpected token. Will not go back when the error is a LexerError.
    pub fn expect_failsafe_wo_eof(&mut self, tok: T)
        -> Result<Option<SpannedTokenOwned<T>>, error::ParseError<T, SpannedTokenOwned<T>>> {
        trace!("{} - expecting [failsafe w/o eof] {:?}", "  ".repeat(self.save_points.len()), tok);

        match self.expect(tok) {
            Ok(res) => {
                trace!("{} - expected [failsafe w/o eof] {:?}", "  ".repeat(self.save_points.len()), &res);

                Ok(Some(res))
            }

            Err(err) => {
                // only go back when the error is recoverable (unexpected token)
                if err.is_recoverable_w_eof() {
                    trace!("{} v unexpected [failsafe w/o eof]", "  ".repeat(self.save_points.len()));
                    self.previous();
                    Ok(None)
                } else {
                    trace!(
                        "{} v unexpected [failsafe w/o eof] irrecoverable err: {}",
                        "  ".repeat(self.save_points.len()), err
                    );
                    Err(err)
                }
            }
        }
    }

    /// Expects if the next token is either of the token specified and return the token; otherwise
    /// it will return a [`error::ParseError::UnexpectedTokenError`]
    pub fn expect_multiple_choices(&mut self, tokens: &[T])
        -> Result<SpannedTokenOwned<T>, error::ParseError<T, SpannedTokenOwned<T>>> {
        self.current_save_point()
            .expect("start() must be called first");

        let next: SpannedTokenOwned<T> = self.next()
            .map_err(|err| err.map_eof_expected(|| Vec::from(tokens)))?
            .clone()
            .into();

        if tokens.contains(&next.token) {
            trace!("{} - expected {:?} (expecting mult {:?})", "  ".repeat(self.save_points.len()), next, tokens);

            Ok(next)
        } else {
            trace!("{} ! unexpected {:?} (expecting mult {:?})", "  ".repeat(self.save_points.len()), next, tokens);

            Err(error::ParseError::UnexpectedTokenError {
                expected: Some(Vec::from(tokens)),
                pos: next.pos.clone(),
                unexpected_token: next.into(),
            })
        }
    }

    /// Peeks and expects if the next token is either of the token specified and return the token;
    /// otherwise it will return a [`error::ParseError::UnexpectedTokenError`].
    pub fn expect_peek_multiple_choices(&mut self, tokens: &[T])
                                   -> Result<SpannedTokenOwned<T>, error::ParseError<T, SpannedTokenOwned<T>>> {
        trace!("{} - peeking and expecting multiple choices", "  ".repeat(self.save_points.len()));

        let res = self.expect_multiple_choices(&tokens);
        self.previous();
        res
    }

    /// Opposite of [`BufferedLexer::expect`]
    pub fn not_expect(&mut self, tok: T)
        -> Result<SpannedTokenOwned<T>, error::ParseError<T, SpannedTokenOwned<T>>> {

        self.current_save_point()
            .expect("start() must be called first");

        let next: SpannedTokenOwned<T> = self.next()?.clone().into();

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
    /// Will panic if previous gets called after a [`BufferedLexer::success`] (it removes all the cache before
    /// it)
    pub fn previous(&mut self) -> Option<&SpannedToken<T>> {
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

    /// Peeks one token ahead. Basically does [`BufferedLexer::next`] and then [`BufferedLexer::previous`].
    pub fn peek(&mut self)
        -> Result<SpannedTokenOwned<T>, error::ParseError<T, SpannedTokenOwned<T>>> {

        self.current_save_point()
            .expect("start() must be called first");

        let next = self.next()?.clone().into();
        self.previous();

        trace!("{} - peeked {:?}", "  ".repeat(self.save_points.len()), next);

        Ok(next)
    }

    /// Peeks one token ahead, and expects a token. Then go back if the error is recoverable
    /// (unexpected token)
    pub fn expect_peek(&mut self, tok: T)
        -> Result<SpannedTokenOwned<T>, error::ParseError<T, SpannedTokenOwned<T>>> {

        trace!("{} - expect_peeking {:?}", "  ".repeat(self.save_points.len()), tok);

        let res = self.expect(tok);

        // only go previous when the error is recoverable (an unexpected token)
        // will not go previous when there is an error token or an EOF
        if let Err(err) = &res {
            if err.is_recoverable() {
                let _ = self.previous().unwrap();
            }
        } else {
            let _ = self.previous().unwrap();
        }

        res
    }

    /// Peeks one token ahead, and not expect a token.
    pub fn not_expect_peek(&mut self, tok: T)
                       -> Result<SpannedTokenOwned<T>, error::ParseError<T, SpannedTokenOwned<T>>> {

        trace!("{} - not_expect_peeking {:?}", "  ".repeat(self.save_points.len()), tok);

        let res = self.not_expect(tok)?;
        let _ = self.previous().unwrap();

        Ok(res)
    }

    /// Blacklists a token on [`BufferedLexer::expect`] or any similar functions.
    /// 
    /// When [`BufferedLexer::expect`] (or any similar functions) gets invoked and encounters any of
    /// the token that got blacklisted, it will skip the token and process the next token.
    pub fn blacklist(&mut self, tok: T) {
        if !self.blacklist.contains(&tok) {
            self.blacklist.push(tok);
        }
    }

    /// Removes a token from the blacklist
    /// 
    /// Returns an [`Err(())`] if the provided token wasn't found on the blacklist.
    pub fn remove_blacklist(&mut self, tok: T) -> Result<(), ()> {
        self.blacklist.remove(
            self.blacklist
                .iter()
                .position(|x| *x == tok)
                .ok_or(())?
        );

        Ok(())
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

    /// Pops the current save point and deletes all the cached tokens before the current index.
    pub fn success(&mut self) {
        // prevents splitting when there are multiple success calls at the end of parsing
        // (since there isn't any tokens left)
        if self.index != self.cache_start_point {
            self.cached_tokens =
                self.cached_tokens.split_off(self.index - self.cache_start_point - 1);
        }

        self.cache_start_point = if self.index != 0 { self.index - 1 } else { 0 };

        self.save_points.pop()
            .expect("Failed to pop the previous save point, is success() called after a start()?");

        trace!("{}<== success ({})", "  ".repeat(self.save_points.len() + 1), self.index);
    }
}

pub mod error {
    use std::error::Error;
    use std::fmt::{Debug, Display, Formatter};

    #[derive(PartialEq, Clone)]
    pub enum ParseError<ExpectedToken: Debug, UnexpectedToken: Debug> {
        // unexpected token
        UnexpectedTokenError {
            expected: Option<Vec<ExpectedToken>>,
            unexpected_token: UnexpectedToken,
            pos: std::ops::Range<usize>,
        },

        // end of file
        EOF {
            expected: Option<Vec<ExpectedToken>>
        },

        // when the lexer wrapper encounters an Error token
        LexerError {
            err_token: ExpectedToken,
            pos: std::ops::Range<usize>,
            slice: String
        }
    }

    /// Propagates non recoverable errors (LexerError, and EOF)
    #[macro_export]
    macro_rules! propagate_non_recoverable {
        ($rule: expr) => {
            match $rule {
                Ok(res) => Ok(res),
                Err(er) => if !er.is_recoverable() { Err(er)? } else { Err(er) }
            }
        };
    }

    /// Propagates non recoverable errors except for EOF (only propagates LexerError)
    #[macro_export]
    macro_rules! propagate_non_recoverable_wo_eof {
        ($rule: expr) => {
            match $rule {
                Ok(res) => Ok(res),
                Err(er) => if !er.is_recoverable_w_eof() { Err(er)? } else { Err(er) }
            }
        };
    }

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
                                    .skip(1)
                                    // safety: .unwrap() because we wouldn't "didn't expect a token" if we don't expect anything.
                                    .fold(format!("{:?}", e.iter().nth(0).unwrap()),
                                        |acc, tok| format!("{} or {:?}", acc, tok)),
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
                                "expected a {:?}, but reached end-of-file",
                                expected.iter()
                                    .fold(String::new(), |acc, tok| {
                                        format!("{:?} or {:?}", acc, tok)
                                    })[..3].to_string(), // removes the trailing ` or`
                            )
                        }
                    } else {
                        write!(f, "reached end-of-file")
                    }
                }
                ParseError::LexerError { err_token, pos, slice } => {
                    write!(f, "lexer error {:?} at {:?}: `{:?}`", err_token, pos, slice)
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