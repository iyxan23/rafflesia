use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use logos::{Lexer, Logos, Source};

#[derive(Debug, Clone, PartialEq)]
pub struct TokenWrapper<'source, T: Debug + Clone + PartialEq> {
    token: T,
    slice: &'source str,
    pos: std::ops::Range<usize>
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
    cached_tokens: VecDeque<TokenWrapper<'source, T>>,
    inner_index: u32,

    save_points: Vec<u32>,
    index: u32,
}

impl<'source, T> LexerWrapper<'source, T>
    where
        T: Logos<'source> + Debug + Clone + PartialEq,
        <<T as Logos<'source>>::Source as Source>::Slice: AsRef<str> {

    pub fn new(inner: Lexer<'source, T>) -> LexerWrapper<'source, T> {
        LexerWrapper {
            inner,
            cached_tokens: VecDeque::new(),
            inner_index: 0,
            save_points: vec![0],
            index: 0,
        }
    }

    fn current_save_point(&self) -> Option<&u32> {
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

            self.cached_tokens.get(self.index as usize)
        } else {
            // nope, this is up-to-date! go next and save it to the cache
            let next_token = self.inner.next()?; // will return None if there is none left

            self.cached_tokens.push_front(TokenWrapper {
                token: next_token,
                slice: self.inner.slice().as_ref(),
                pos: self.inner.span(),
            });

            self.index += 1;
            self.inner_index += 1;

            Some(&self.cached_tokens.front().unwrap())
        }
    }

    // /// Do a check on the next token and if `f` returns true, it will return `Ok(tok)`. otherwise
    // /// it will return an `Err()` with the message returned by the given `err` function.
    // pub fn expect_fn_err<F, UEF>(&mut self, check: F, unexpected_tok_err: UEF)
    //                              -> Result<&TokenWrapper<T>, error::ParseError<T>>
    //     where
    //         F: FnOnce(&T) -> bool,
    //         UEF: FnOnce(&TokenWrapper<T>) -> String {
    //
    //     self.next().ok_or_else(||)
    // }
    //
    // pub fn expect(&mut self, tok: &T) -> Result<&TokenWrapper<T>, error::ParseError<T>> {
    //     self.expect_fn(|t| t == tok)
    // }

    pub fn previous(&mut self) -> Option<&T> {
        let state_start_point = self.current_save_point()
            .expect("start() must be called first");

        // check if it's trying to get the token before its save point
        if self.index - 1 < *state_start_point {
            panic!("trying to access tokens out of bounds");
        }

        // gogogogo
        self.index -= 1;

        self.cached_tokens.get(self.index as usize)
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

    /// Removes the current save point
    pub fn success(&mut self) {
        self.save_points.pop()
            .expect("Failed to pop the previous save point, is success() called after a start()?");
    }
}

pub mod error {
    use std::error::Error;
    use std::fmt::{Debug, Display, Formatter};

    pub enum ParseError<T> {
        // unexpected token
        UnexpectedTokenError {
            expected: T,
            unexpected_token: T,
            range: std::ops::Range<usize>,

            // should usually be something like
            // expected token {}, got {} instead
            message: String,
        },

        // end of file
        EOF
    }

    impl<T> Debug for ParseError<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                ParseError::UnexpectedTokenError { message, .. } => write!(f, "{}", message),
                ParseError::EOF => write!(f, "reached end-of-file")
            }
        }
    }

    impl<T> Display for ParseError<T> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            <Self as Debug>::fmt(self, f)
        }
    }

    impl<T> Error for ParseError<T> {}
}