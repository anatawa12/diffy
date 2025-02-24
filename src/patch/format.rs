use super::{Hunk, Line, Patch, NO_NEWLINE_AT_EOF};
#[cfg(feature = "ansi_term")]
use ansi_term::{Color, Style};
use std::{
    fmt::{Display, Formatter, Result},
    io,
};

/// Struct used to adjust the formatting of a `Patch`
#[derive(Debug)]
pub struct PatchFormatter {
    #[cfg(feature = "ansi_term")]
    with_color: bool,
    with_space_on_empty_line: bool,

    #[cfg(feature = "ansi_term")]
    context: Style,
    #[cfg(feature = "ansi_term")]
    delete: Style,
    #[cfg(feature = "ansi_term")]
    insert: Style,
    #[cfg(feature = "ansi_term")]
    hunk_header: Style,
    #[cfg(feature = "ansi_term")]
    patch_header: Style,
    #[cfg(feature = "ansi_term")]
    function_context: Style,
}

impl PatchFormatter {
    /// Construct a new formatter
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "ansi_term")]
            with_color: false,
            with_space_on_empty_line: false,

            #[cfg(feature = "ansi_term")]
            context: Style::new(),
            #[cfg(feature = "ansi_term")]
            delete: Color::Red.normal(),
            #[cfg(feature = "ansi_term")]
            insert: Color::Green.normal(),
            #[cfg(feature = "ansi_term")]
            hunk_header: Color::Cyan.normal(),
            #[cfg(feature = "ansi_term")]
            patch_header: Style::new().bold(),
            #[cfg(feature = "ansi_term")]
            function_context: Style::new(),
        }
    }

    #[cfg(feature = "ansi_term")]
    /// Enable formatting a patch with color
    pub fn with_color(mut self) -> Self {
        self.with_color = true;
        self
    }

    /// Enable formatting a patch with color
    pub fn with_space_on_empty_line(mut self) -> Self {
        self.with_space_on_empty_line = true;
        self
    }

    /// Returns a `Display` impl which can be used to print a Patch
    pub fn fmt_patch<'a>(&'a self, patch: &'a Patch<'a, str>) -> impl Display + 'a {
        PatchDisplay { f: self, patch }
    }

    pub fn write_patch_into<T: ToOwned + AsRef<[u8]> + ?Sized, W: io::Write>(
        &self,
        patch: &Patch<'_, T>,
        w: W,
    ) -> io::Result<()> {
        PatchDisplay { f: self, patch }.write_into(w)
    }

    pub fn fmt_hunk<'a>(&'a self, hunk: &'a Hunk<'a, str>) -> impl Display + 'a {
        HunkDisplay { f: self, hunk }
    }

    pub fn write_hunk_into<T: AsRef<[u8]> + ?Sized, W: io::Write>(
        &self,
        hunk: &Hunk<'_, T>,
        w: W,
    ) -> io::Result<()> {
        HunkDisplay { f: self, hunk }.write_into(w)
    }

    fn fmt_line<'a>(&'a self, line: &'a Line<'a, str>) -> impl Display + 'a {
        LineDisplay { f: self, line }
    }

    fn write_line_into<T: AsRef<[u8]> + ?Sized, W: io::Write>(
        &self,
        line: &Line<'_, T>,
        w: W,
    ) -> io::Result<()> {
        LineDisplay { f: self, line }.write_into(w)
    }
}

impl Default for PatchFormatter {
    fn default() -> Self {
        Self::new()
    }
}

struct PatchDisplay<'a, T: ToOwned + ?Sized> {
    f: &'a PatchFormatter,
    patch: &'a Patch<'a, T>,
}

impl<T: ToOwned + AsRef<[u8]> + ?Sized> PatchDisplay<'_, T> {
    fn write_into<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        if self.patch.original.is_some() || self.patch.modified.is_some() {
            #[cfg(feature = "ansi_term")]
            if self.f.with_color {
                write!(w, "{}", self.f.patch_header.prefix())?;
            }
            if let Some(original) = &self.patch.original {
                write!(w, "--- ")?;
                original.write_into(&mut w)?;
                writeln!(w)?;
            }
            if let Some(modified) = &self.patch.modified {
                write!(w, "+++ ")?;
                modified.write_into(&mut w)?;
                writeln!(w)?;
            }
            #[cfg(feature = "ansi_term")]
            if self.f.with_color {
                write!(w, "{}", self.f.patch_header.suffix())?;
            }
        }

        for hunk in &self.patch.hunks {
            self.f.write_hunk_into(hunk, &mut w)?;
        }

        Ok(())
    }
}

impl Display for PatchDisplay<'_, str> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if self.patch.original.is_some() || self.patch.modified.is_some() {
            #[cfg(feature = "ansi_term")]
            if self.f.with_color {
                write!(f, "{}", self.f.patch_header.prefix())?;
            }
            if let Some(original) = &self.patch.original {
                writeln!(f, "--- {}", original)?;
            }
            if let Some(modified) = &self.patch.modified {
                writeln!(f, "+++ {}", modified)?;
            }
            #[cfg(feature = "ansi_term")]
            if self.f.with_color {
                write!(f, "{}", self.f.patch_header.suffix())?;
            }
        }

        for hunk in &self.patch.hunks {
            write!(f, "{}", self.f.fmt_hunk(hunk))?;
        }

        Ok(())
    }
}

struct HunkDisplay<'a, T: ?Sized> {
    f: &'a PatchFormatter,
    hunk: &'a Hunk<'a, T>,
}

impl<T: AsRef<[u8]> + ?Sized> HunkDisplay<'_, T> {
    fn write_into<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        #[cfg(feature = "ansi_term")]
        if self.f.with_color {
            write!(w, "{}", self.f.hunk_header.prefix())?;
        }
        write!(w, "@@ -{} +{} @@", self.hunk.old_range, self.hunk.new_range)?;
        #[cfg(feature = "ansi_term")]
        if self.f.with_color {
            write!(w, "{}", self.f.hunk_header.suffix())?;
        }

        if let Some(ctx) = self.hunk.function_context {
            write!(w, " ")?;
            #[cfg(feature = "ansi_term")]
            if self.f.with_color {
                write!(w, "{}", self.f.function_context.prefix())?;
            }
            write!(w, " ")?;
            w.write_all(ctx.as_ref())?;
            #[cfg(feature = "ansi_term")]
            if self.f.with_color {
                write!(w, "{}", self.f.function_context.suffix())?;
            }
        }
        writeln!(w)?;

        for line in &self.hunk.lines {
            self.f.write_line_into(line, &mut w)?;
        }

        Ok(())
    }
}

impl Display for HunkDisplay<'_, str> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        #[cfg(feature = "ansi_term")]
        if self.f.with_color {
            write!(f, "{}", self.f.hunk_header.prefix())?;
        }
        write!(f, "@@ -{} +{} @@", self.hunk.old_range, self.hunk.new_range)?;
        #[cfg(feature = "ansi_term")]
        if self.f.with_color {
            write!(f, "{}", self.f.hunk_header.suffix())?;
        }

        if let Some(ctx) = self.hunk.function_context {
            write!(f, " ")?;
            #[cfg(feature = "ansi_term")]
            if self.f.with_color {
                write!(f, "{}", self.f.function_context.prefix())?;
            }
            write!(f, " {}", ctx)?;
            #[cfg(feature = "ansi_term")]
            if self.f.with_color {
                write!(f, "{}", self.f.function_context.suffix())?;
            }
        }
        writeln!(f)?;

        for line in &self.hunk.lines {
            write!(f, "{}", self.f.fmt_line(line))?;
        }

        Ok(())
    }
}

struct LineDisplay<'a, T: ?Sized> {
    f: &'a PatchFormatter,
    line: &'a Line<'a, T>,
}

impl<T: AsRef<[u8]> + ?Sized> LineDisplay<'_, T> {
    fn write_into<W: io::Write>(&self, mut w: W) -> io::Result<()> {
        #[cfg(feature = "ansi_term")]
        let (sign, line, style) = match self.line {
            Line::Context(line) => (' ', line.as_ref(), self.f.context),
            Line::Delete(line) => ('-', line.as_ref(), self.f.delete),
            Line::Insert(line) => ('+', line.as_ref(), self.f.insert),
        };
        #[cfg(not(feature = "ansi_term"))]
        let (sign, line) = match self.line {
            Line::Context(line) => (' ', line.as_ref()),
            Line::Delete(line) => ('-', line.as_ref()),
            Line::Insert(line) => ('+', line.as_ref()),
        };

        #[cfg(feature = "ansi_term")]
        if self.f.with_color {
            write!(w, "{}", style.prefix())?;
        }

        if !self.f.with_space_on_empty_line && sign == ' ' && line == b"\n" {
            w.write_all(line)?;
        } else {
            write!(w, "{}", sign)?;
            w.write_all(line)?;
        }

        #[cfg(feature = "ansi_term")]
        if self.f.with_color {
            write!(w, "{}", style.suffix())?;
        }

        if !line.ends_with(b"\n") {
            writeln!(w)?;
            writeln!(w, "{}", NO_NEWLINE_AT_EOF)?;
        }

        Ok(())
    }
}

impl Display for LineDisplay<'_, str> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        #[cfg(feature = "ansi_term")]
        let (sign, line, style) = match self.line {
            Line::Context(line) => (' ', line, self.f.context),
            Line::Delete(line) => ('-', line, self.f.delete),
            Line::Insert(line) => ('+', line, self.f.insert),
        };
        #[cfg(not(feature = "ansi_term"))]
        let (sign, line) = match self.line {
            Line::Context(line) => (' ', line),
            Line::Delete(line) => ('-', line),
            Line::Insert(line) => ('+', line),
        };

        #[cfg(feature = "ansi_term")]
        if self.f.with_color {
            write!(f, "{}", style.prefix())?;
        }

        if sign == ' ' && *line == "\n" {
            write!(f, "{}", line)?;
        } else {
            write!(f, "{}{}", sign, line)?;
        }

        #[cfg(feature = "ansi_term")]
        if self.f.with_color {
            write!(f, "{}", style.suffix())?;
        }

        if !line.ends_with('\n') {
            writeln!(f)?;
            writeln!(f, "{}", NO_NEWLINE_AT_EOF)?;
        }

        Ok(())
    }
}
