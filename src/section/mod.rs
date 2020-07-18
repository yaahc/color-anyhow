//! Helpers for adding custom sections to error reports
use std::fmt::{self, Display, Write};

pub(crate) mod help;

/// An indenteted section with a header for an error report
///
/// # Details
///
/// This helper provides two functions to help with constructing nicely formatted
/// error reports. First, it handles indentation of every line of the body for
/// you, and makes sure it is consistent with the rest of color-anyhow's output.
/// Second, it omits outputting the header if the body itself is empty,
/// preventing unnecessary pollution of the report for sections with dynamic
/// content.
///
/// # Examples
///
/// ```rust
/// use color_anyhow::{anyhow::anyhow, SectionExt, Section, anyhow::Error};
/// use std::process::Command;
/// use tracing::instrument;
///
/// trait Output {
///     fn output2(&mut self) -> Result<String, Error>;
/// }
///
/// impl Output for Command {
///     #[instrument]
///     fn output2(&mut self) -> Result<String, Error> {
///         let output = self.output()?;
///
///         let stdout = String::from_utf8_lossy(&output.stdout);
///
///         if !output.status.success() {
///             let stderr = String::from_utf8_lossy(&output.stderr);
///             Err(anyhow!("cmd exited with non-zero status code"))
///                 .with_section(move || stdout.trim().to_string().header("Stdout:"))
///                 .with_section(move || stderr.trim().to_string().header("Stderr:"))
///         } else {
///             Ok(stdout.into())
///         }
///     }
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct IndentedSection<H, B> {
    header: H,
    body: B,
}

impl<H, B> fmt::Display for IndentedSection<H, B>
where
    H: Display + Send + Sync + 'static,
    B: Display + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut headered = crate::writers::HeaderWriter {
            inner: f,
            header: &self.header,
            started: false,
        };

        let mut headered = crate::writers::HeaderWriter {
            inner: headered.ready(),
            header: &"\n",
            started: false,
        };

        let mut headered = headered.ready();

        let mut indented = indenter::indented(&mut headered)
            .with_format(indenter::Format::Uniform { indentation: "   " });

        write!(&mut indented, "{}", self.body)?;

        Ok(())
    }
}

/// Extension trait for constructing sections with commonly used formats
pub trait SectionExt: Sized {
    /// Add a header to a `Section` and indent the body
    ///
    /// # Details
    ///
    /// Bodies are always indented to the same level as error messages and spans.
    /// The header is not printed if the display impl of the body produces no
    /// output.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use color_anyhow::{anyhow::anyhow, Section, SectionExt, anyhow::Error};
    ///
    /// let all_in_header = "header\n   body\n   body";
    /// let report = Err::<(), Error>(anyhow!("an error occurred"))
    ///     .section(all_in_header)
    ///     .unwrap_err();
    ///
    /// let just_header = "header";
    /// let just_body = "body\nbody";
    /// let report2 = Err::<(), Error>(anyhow!("an error occurred"))
    ///     .section(just_body.header(just_header))
    ///     .unwrap_err();
    ///
    /// assert_eq!(format!("{:?}", report), format!("{:?}", report2))
    /// ```
    fn header<C>(self, header: C) -> IndentedSection<C, Self>
    where
        C: Display + Send + Sync + 'static;
}

impl<T> SectionExt for T
where
    T: Display + Send + Sync + 'static,
{
    fn header<C>(self, header: C) -> IndentedSection<C, Self>
    where
        C: Display + Send + Sync + 'static,
    {
        IndentedSection { body: self, header }
    }
}

/// A helper trait for attaching informational sections to error reports to be
/// displayed after the chain of errors
///
/// # Details
///
/// `color_anyhow` provides two types of help text that can be attached to error reports: custom
/// sections and pre-configured sections. Custom sections are added via the `section` and
/// `with_section` methods, and give maximum control over formatting.
///
/// The pre-configured sections are provided via `suggestion`, `warning`, and `note`. These
/// sections are displayed after all other sections with no extra newlines between subsequent Section
/// sections. They consist only of a header portion and are prepended with a colored string
/// indicating the kind of section, e.g. `Note: This might have failed due to ..."
pub trait Section<T>: crate::private::Sealed {
    /// Add a section to an error report, to be displayed after the chain of errors.
    ///
    /// # Details
    ///
    /// Sections are displayed in the order they are added to the error report. They are displayed
    /// immediately after the `Error:` section and before the `SpanTrace` and `Backtrace` sections.
    /// They consist of a header and an optional body. The body of the section is indented by
    /// default.
    ///
    /// # Examples
    ///
    /// ```rust,should_panic
    /// use color_anyhow::{anyhow::anyhow, anyhow::Error, Section};
    ///
    /// Err(anyhow!("command failed"))
    ///     .section("Please report bugs to https://real.url/bugs")?;
    /// # Ok::<_, Error>(())
    /// ```
    fn section<D>(self, section: D) -> anyhow::Result<T>
    where
        D: Display + Send + Sync + 'static;

    /// Add a Section to an error report, to be displayed after the chain of errors. The closure to
    /// create the Section is lazily evaluated only in the case of an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use color_anyhow::{anyhow::anyhow, anyhow::Error, Section, SectionExt};
    ///
    /// let output = std::process::Command::new("ls")
    ///     .output()?;
    ///
    /// let output = if !output.status.success() {
    ///     let stderr = String::from_utf8_lossy(&output.stderr);
    ///     Err(anyhow!("cmd exited with non-zero status code"))
    ///         .with_section(move || stderr.trim().to_string().header("Stderr:"))?
    /// } else {
    ///     String::from_utf8_lossy(&output.stdout)
    /// };
    ///
    /// println!("{}", output);
    /// # Ok::<_, Error>(())
    /// ```
    fn with_section<D, F>(self, section: F) -> anyhow::Result<T>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;

    /// Add an error section to an error report, to be displayed after the primary error message
    /// section.
    ///
    /// # Examples
    ///
    /// ```rust,should_panic
    /// use color_anyhow::{anyhow::anyhow, anyhow::Error, Section};
    /// use thiserror::Error;
    ///
    /// #[derive(Debug, Error)]
    /// #[error("{0}")]
    /// struct StrError(&'static str);
    ///
    /// Err(anyhow!("command failed"))
    ///     .error(StrError("got one error"))
    ///     .error(StrError("got a second error"))?;
    /// # Ok::<_, Error>(())
    /// ```
    fn error<E>(self, error: E) -> anyhow::Result<T>
    where
        E: std::error::Error + Send + Sync + 'static;

    /// Add an error section to an error report, to be displayed after the primary error message
    /// section. The closure to create the Section is lazily evaluated only in the case of an error.
    ///
    /// # Examples
    ///
    /// ```rust,should_panic
    /// use color_anyhow::{anyhow::anyhow, anyhow::Error, Section};
    /// use thiserror::Error;
    ///
    /// #[derive(Debug, Error)]
    /// #[error("{0}")]
    /// struct StringError(String);
    ///
    /// Err(anyhow!("command failed"))
    ///     .with_error(|| StringError("got one error".into()))
    ///     .with_error(|| StringError("got a second error".into()))?;
    /// # Ok::<_, Error>(())
    /// ```
    fn with_error<E, F>(self, error: F) -> anyhow::Result<T>
    where
        F: FnOnce() -> E,
        E: std::error::Error + Send + Sync + 'static;

    /// Add a Note to an error report, to be displayed after the chain of errors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::{error::Error, fmt::{self, Display}};
    /// # use color_anyhow::anyhow::Result;
    /// # #[derive(Debug)]
    /// # struct FakeErr;
    /// # impl Display for FakeErr {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "FakeErr")
    /// #     }
    /// # }
    /// # impl std::error::Error for FakeErr {}
    /// # fn main() -> Result<()> {
    /// # fn fallible_fn() -> Result<(), FakeErr> {
    /// #       Ok(())
    /// # }
    /// use color_anyhow::Section as _;
    ///
    /// fallible_fn().note("This might have failed due to ...")?;
    /// # Ok(())
    /// # }
    /// ```
    fn note<D>(self, note: D) -> anyhow::Result<T>
    where
        D: Display + Send + Sync + 'static;

    /// Add a Note to an error report, to be displayed after the chain of errors. The closure to
    /// create the Note is lazily evaluated only in the case of an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::{error::Error, fmt::{self, Display}};
    /// # use color_anyhow::anyhow::Result;
    /// # #[derive(Debug)]
    /// # struct FakeErr;
    /// # impl Display for FakeErr {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "FakeErr")
    /// #     }
    /// # }
    /// # impl std::error::Error for FakeErr {}
    /// # fn main() -> Result<()> {
    /// # fn fallible_fn() -> Result<(), FakeErr> {
    /// #       Ok(())
    /// # }
    /// use color_anyhow::Section as _;
    ///
    /// fallible_fn().with_note(|| {
    ///         format!("This might have failed due to ... It has failed {} times", 100)
    ///     })?;
    /// # Ok(())
    /// # }
    /// ```
    fn with_note<D, F>(self, f: F) -> anyhow::Result<T>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;

    /// Add a Warning to an error report, to be displayed after the chain of errors.
    fn warning<D>(self, warning: D) -> anyhow::Result<T>
    where
        D: Display + Send + Sync + 'static;

    /// Add a Warning to an error report, to be displayed after the chain of errors. The closure to
    /// create the Warning is lazily evaluated only in the case of an error.
    fn with_warning<D, F>(self, f: F) -> anyhow::Result<T>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;

    /// Add a Suggestion to an error report, to be displayed after the chain of errors.
    fn suggestion<D>(self, suggestion: D) -> anyhow::Result<T>
    where
        D: Display + Send + Sync + 'static;

    /// Add a Suggestion to an error report, to be displayed after the chain of errors. The closure
    /// to create the Suggestion is lazily evaluated only in the case of an error.
    fn with_suggestion<D, F>(self, f: F) -> anyhow::Result<T>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;
}
