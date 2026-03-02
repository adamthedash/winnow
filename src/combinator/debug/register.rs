//!
//! These only have an effect when compiling with debug feature
//! `WINNOW_LOG=all`      (default)
//! `WINNOW_LOG=poe_data_tools` (enable for a crate)
//! `WINNOW_LOG=winnow`
//! `WINNOW_LOG=`            (disable all logging)
//! `WINNOW_LOG=poe_data_tools::file_parsers::ao` (enable for a module)
//! `WINNOW_LOG=poe_data_tools::file_parsers::ao=children` (enable for a parser and all child
//! parsers called by it)
//! `WINNOW_LOG=winnow,poe_data_tools::file_parsers` (multiple filters)
//!
//!

use lazy_static::lazy_static;
use std::fmt::Display;

lazy_static! {
    pub(crate) static ref FILTERS: Filters = Filters(Filter::from_env());
}

#[derive(Debug, PartialEq, Eq)]
enum Filter {
    All,
    Id(String),
}

impl Filter {
    /// Whether the provided indentifier should be allowed
    /// Identifier should be a fully qualified path to the parser.
    /// Eg `winnow::combinators::preceded`
    fn enabled(&self, identifier: &str) -> bool {
        match self {
            Filter::All => true,
            // TODO: Structured path?
            Filter::Id(id) => identifier.starts_with(id),
        }
    }

    fn parse_env(env: &str) -> Vec<Self> {
        // TODO: Validation
        env.split(',')
            .map(|part| part.trim())
            .filter(|part| !part.is_empty())
            .map(|part| {
                if part == "all" {
                    Filter::All
                } else {
                    Filter::Id(part.to_owned())
                }
            })
            .collect()
    }

    fn from_env() -> Vec<Self> {
        let Ok(env_str) = std::env::var("WINNOW_TRACE") else {
            // ENV not set, so let everything through
            return vec![Self::All];
        };

        Self::parse_env(&env_str)
    }
}

pub(crate) struct Filters(Vec<Filter>);

impl Filters {
    pub(crate) fn enabled(&self, identifier: impl Display) -> bool {
        let identifier = format!("{identifier}");
        self.0.iter().any(|f| f.enabled(&identifier))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_all() {
        let filter = Filter::All;

        assert!(filter.enabled("winnow::combinators::preceded"));
        assert!(filter.enabled("winnow::combinators::terminated"));
        assert!(filter.enabled("winnow::mutli::repeat"));
        assert!(filter.enabled("winnow::binary::be_u16"));
        assert!(filter.enabled("other_crate::parser"));
    }

    #[test]
    fn test_filter_crate() {
        let filter = Filter::Id("winnow".to_owned());

        assert!(filter.enabled("winnow::combinators::preceded"));
        assert!(filter.enabled("winnow::combinators::terminated"));
        assert!(filter.enabled("winnow::mutli::repeat"));
        assert!(filter.enabled("winnow::binary::be_u16"));
        assert!(!filter.enabled("other_crate::parser"));
    }

    #[test]
    fn test_filter_module() {
        let filter = Filter::Id("winnow::combinators".to_owned());

        assert!(filter.enabled("winnow::combinators::preceded"));
        assert!(filter.enabled("winnow::combinators::terminated"));
        assert!(!filter.enabled("winnow::mutli::repeat"));
        assert!(!filter.enabled("winnow::binary::be_u16"));
        assert!(!filter.enabled("other_crate::parser"));
    }

    #[test]
    fn test_env_filters_all() {
        let env_str = "all";

        let filters = Filter::parse_env(env_str);

        assert_eq!(filters, [Filter::All]);
    }

    #[test]
    fn test_env_filters_multi() {
        let env_str = "winnow::combinator,winnow::combinator::multi::repeat";

        let filters = Filter::parse_env(env_str);

        assert_eq!(
            filters,
            [
                Filter::Id("winnow::combinator".to_owned()),
                Filter::Id("winnow::combinator::multi::repeat".to_owned())
            ]
        );
    }

    #[test]
    fn test_env_filters_none() {
        let env_str = "";

        let filters = Filter::parse_env(env_str);

        assert_eq!(filters, []);
    }

    #[test]
    fn test_filters_all() {
        let filter = Filters(vec![Filter::All]);

        assert!(filter.enabled("winnow::combinators::preceded"));
        assert!(filter.enabled("winnow::combinators::terminated"));
        assert!(filter.enabled("winnow::mutli::repeat"));
        assert!(filter.enabled("winnow::binary::be_u16"));
        assert!(filter.enabled("other_crate::parser"));
    }

    #[test]
    fn test_filters_multi() {
        let filter = Filters(vec![
            Filter::Id("winnow::combinators".to_owned()),
            Filter::Id("winnow::binary::be_u16".to_owned()),
        ]);

        assert!(filter.enabled("winnow::combinators::preceded"));
        assert!(filter.enabled("winnow::combinators::terminated"));
        assert!(!filter.enabled("winnow::mutli::repeat"));
        assert!(filter.enabled("winnow::binary::be_u16"));
        assert!(!filter.enabled("other_crate::parser"));
    }

    #[test]
    fn test_filters_none() {
        let filter = Filters(vec![]);

        assert!(!filter.enabled("winnow::combinators::preceded"));
        assert!(!filter.enabled("winnow::combinators::terminated"));
        assert!(!filter.enabled("winnow::mutli::repeat"));
        assert!(!filter.enabled("winnow::binary::be_u16"));
        assert!(!filter.enabled("other_crate::parser"));
    }
}
