#[derive(Debug, Clone)]
pub struct MultiResult<T, E> {
    ok: Vec<T>,
    err: Vec<E>,
}

impl<T, E> MultiResult<T, E> {
    /// Create a new [MultiResult]
    pub fn new() -> Self {
        Self {
            ok: Vec::new(),
            err: Vec::new(),
        }
    }
    /// Composes a [MultiResult] from ok and err [Vec] respectively
    pub fn compose(ok: Vec<T>, err: Vec<E>) -> Self {
        Self {
            ok,
            err,
        }
    }
    /// Push an ok element
    pub fn push_ok(&mut self, t: T) {
        self.ok.push(t);
    }
    /// Push an err element
    pub fn push_err(&mut self, e: E) {
        self.err.push(e);
    }
    /// Push from a result 
    pub fn push(&mut self, res: Result<T, E>) {
        match res {
            Ok(v) => self.push_ok(v),
            Err(e) => self.push_err(e),
        }
    }
    /// Returns if there is at least one ok element
    pub fn has_ok(&self) -> bool {
        !self.ok.is_empty()
    }
    /// Returns if there is at least one err element
    pub fn has_err(&self) -> bool {
        !self.err.is_empty()
    }
    /// Consume the result and return the ok [Vec]
    pub fn ok(self) -> Vec<T> {
        self.ok
    }
    /// Consume the result and return the err [Vec]
    pub fn err(self) -> Vec<E> {
        self.err
    }
    /// Consume the result and return the ok and err [Vec]
    pub fn extract(self) -> (Vec<T>, Vec<E>) {
        (self.ok, self.err)
    }
    /// Maps all ok elements and return the new [MultiResult]
    pub fn map<F, U>(self, f: F) -> MultiResult<U, E> 
        where F: Fn(T) -> U,
    {
        MultiResult {
            ok: self.ok.into_iter().map(f).collect(),
            err: self.err,
        }
    }
    /// Maps all err elements and return the new [MultiResult]
    pub fn map_err<F, U>(self, f: F) -> MultiResult<T, U>
        where F: Fn(E) -> U,
    {
        MultiResult {
            ok: self.ok,
            err: self.err.into_iter().map(f).collect(),
        }
    }
}
impl<T, E> Default for MultiResult<T, E> {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Debug)]
pub enum CategoryError {
    BadID(u64),
    SeaORM(sea_orm::DbErr),
}
pub type CategoryResult<T> = Result<T, CategoryError>;
pub type CategoriesResult<T> = MultiResult<T, CategoryError>;

#[derive(Debug)]
pub enum UserError {
    BadID(u64),
    SeaORM(sea_orm::DbErr),
}
pub type UserResult<T> = Result<T, UserError>;
pub type UsersResult<T> = MultiResult<T, UserError>;

#[derive(Debug)]
pub enum MessageError {
    BadID(u64),
    BadUserID(u64),
    BadReplyID(u64),
    SeaORM(sea_orm::DbErr),
}
pub type MessageResult<T> = Result<T, MessageError>;
pub type MessagesResult<T> = MultiResult<T, MessageError>;

#[derive(Debug)]
pub enum ChannelError {
    BadID(u64),
    SeaORM(sea_orm::DbErr),
    NotFoundAfterInsert
}
pub type ChannelResult<T> = Result<T, ChannelError>;
pub type ChannelsResult<T> = MultiResult<T, ChannelError>;

pub enum FileError {
    Io(std::io::Error),
    Serde(serde_json::Error),
    NotFound(std::path::PathBuf),
}

pub type FileResult<T> = Result<T, FileError>;

pub enum ArchiveError {
    File(FileError),
    Channel(ChannelError),
    ClosedBy(UserError),
    SeaORM(sea_orm::DbErr),
}

pub type ArchiveResult<T> = Result<T, ArchiveError>;
pub type ArchivesResult<T> = MultiResult<T, ArchiveError>;

pub enum MigrationError {
    DataTickets(FileError),
    Archives(FileError)
}

pub type MigrationResult<T> = Result<T, MigrationError>;