//! doc

use std::{error::Error, io::stderr, string};

use snafu::ErrorCompat;

#[cfg(test)]
mod test {
    use std::error::Error;
    use std::io::{Error as IOError, ErrorKind as IOErrorKind, Result as IOResult, Write, stderr};
    use std::result::Result;

    #[test]
    fn fork_join_example() {
        use std::thread;
        fn task(panic: bool) -> IOResult<usize> {
            if panic {
                panic!("i am panicking");
            }
            Ok(0_usize)
        }
        for (i, handle) in
            [thread::spawn(|| task(true)), thread::spawn(|| task(false))].into_iter().enumerate()
        {
            // if let Err(err) = handle.join() {
            //     println!("child thread {} panicked", i);
            // }
            if let Ok(ret) = handle.join() {
                println!("child thread {} returns", i);
                if let Ok(value) = ret {
                    println!("child thread {} application error: {}", i, value);
                }
            } else {
                println!("child thread {} panicked", i);
            }
        }
    }

    #[test]
    fn print_error() {
        /// Dump an error message to `stderr`.
        /// If another error happens while building the error message or
        /// writing to `stderr`, it is ignored.
        fn print_error(mut err: &dyn Error) {
            let _ = writeln!(stderr(), "print_error: {err}");
            while let Some(source) = err.source() {
                let _ = writeln!(stderr(), "caused by: {source}");
                err = source;
            }
        }

        let err = Box::new(IOError::new(IOErrorKind::InvalidInput, "InvalidInput"));

        let err: &dyn Error = err.as_ref();
        print_error(err);
    }
}

#[test]
fn enrich_error() {
    use snafu::{Location, ResultExt, Snafu, location};
    use std::error::Error;
    use std::io;
    use std::io::Write;

    #[derive(Debug, Snafu)]
    pub enum UserRepositoryError {
        // business errors
        #[snafu(display("{location}: UserNotFound [user_id:{user_id}]"))]
        UserNotFound {
            user_id: String,
            #[snafu(implicit)]
            location: Location,
        },
        #[snafu(display("{location}: UpdateUserThatNotExisted [user_id:{user_id}]"))]
        UpdateUserThatNotExisted {
            user_id: String,
            #[snafu(implicit)]
            location: Location,
        },
        #[snafu(display("{location}: CreateUserThatExisted [user_id:{user_id}]"))]
        CreateUserThatExisted {
            user_id: String,
            #[snafu(implicit)]
            location: Location,
        },

        // wrapper error type caused by underlying database errors
        #[snafu(display("{location}: UserQueryFailure"))]
        UserQueryFailure {
            source: DatabaseError,
            #[snafu(implicit)]
            location: Location,
        },
    }

    #[derive(Debug, Snafu)]
    pub enum ConnectionError {
        #[snafu(display("{location}: ConnectionAuthFailure"))]
        AuthFailure {
            connected_at_time: usize,
            peer_address: String,
            auth_principal: String, //user-id / user-name
            auth_type: String,      // username-pwd,sso, otp
            #[snafu(implicit)]
            location: Location,
        },

        #[snafu(display("{location}: Disconnected"))]
        Disconnected {
            disconnected_at_time: usize,
            peer_address: String,
            source: std::io::Error,
            #[snafu(implicit)]
            location: Location,
        },

        #[snafu(display("{location}: ConnectTimeout"))]
        ConnectTimeout {
            connect_at_time: usize,
            connect_timeout_at_time: usize,
            peer_address: String,
            #[snafu(implicit)]
            location: Location,
        },
    }

    #[derive(Debug, Snafu)]
    pub enum DatabaseError {
        #[snafu(display("{location}: DatabaseConnectionFailed"))]
        DBConnectionFailure {
            db_connection_url: String,
            db_connection_timeout_period: u32,
            db_connection_retry_times: u16,
            db_connection_extra: String,
            source: ConnectionError,
            #[snafu(implicit)]
            location: Location,
        },

        #[snafu(display("{location}: InvalidQuerySyntax query[{query}]"))]
        SyntaxError {
            query: String,
            #[snafu(implicit)]
            location: Location,
        },

        #[snafu(display("{location}: DatabaseTimeout"))]
        Timeout {
            #[snafu(implicit)]
            location: Location,
        },

        #[snafu(display("{location}: DatabaseTransactionFailure"))]
        Transaction {
            #[snafu(implicit)]
            location: Location,
        },
    }

    pub type UserResult<T> = std::result::Result<T, UserRepositoryError>;
    pub type DatabaseResult<T> = std::result::Result<T, DatabaseError>;

    struct User {
        id: String,
        name: String,
    }

    // Mock database client
    struct DatabaseClient;

    impl DatabaseClient {
        fn connect(url: &str) -> Result<Self, io::Error> {
            // Simulate connection that might fail with IO error
            Ok(DatabaseClient)
        }

        fn query_user(&self, user_id: &str) -> DatabaseResult<Option<User>> {
            // Simulate database operations that might fail
            if user_id == "timeout" {
                return Err(DatabaseError::Timeout {
                    location: location!(),
                });
            }

            if user_id == "invalid" {
                return Err(DatabaseError::SyntaxError {
                    query: "SELECT * FROM users".to_string(),
                    location: location!(),
                });
            }

            // Simulate finding/not finding user
            if user_id == "123" {
                Ok(Some(User {
                    id: user_id.to_string(),
                    name: "Alice".to_string(),
                }))
            } else {
                Ok(None)
            }
        }
    }

    pub fn get_user(user_id: &str) -> UserResult<User> {
        let db = DatabaseClient::connect("localhost:1234")
            .with_context(|_| ConnectionSnafu)
            .with_context(|_| QueryFailureSnafu)?;
        let db = db.query_user(user_id).with_context(|_| QueryFailureSnafu)?;
        db.ok_or_else(|| UserRepositoryError::UserNotFound {
            user_id: user_id.to_string(),
            location: location!(),
        })
    }

    fn print_error_chain(mut err: &dyn Error) {
        let _ = writeln!(io::stderr(), "{err}");
        while let Some(source) = err.source() {
            writeln!(io::stderr(), "Caused by: {source}");
            err = source;
        }
    }

    // Successful case
    match get_user("123") {
        Ok(user) => println!("Found user: {}", user.name),
        Err(e) => println!("Error: {}", e),
    }

    // User not found
    match get_user("456") {
        Ok(user) => println!("Found user: {}", user.name),
        Err(e) => {
            print_error_chain(&e as &dyn Error);
        }
    }

    // Database timeout
    match get_user("timeout") {
        Ok(user) => println!("Found user: {}", user.name),
        Err(e) => {
            print_error_chain(&e);
            // if let Some(backtrace) = e.backtrace() {
            //     println!("\nBacktrace:\n{:#?}", backtrace);
            // }
        }
    }

    // Database invlid query
    match get_user("invalid") {
        Ok(user) => println!("Found user: {}", user.name),
        Err(e) => {
            print_error_chain(&e);
        }
    }
}

fn main() {}
