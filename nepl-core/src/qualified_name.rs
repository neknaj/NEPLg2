pub(crate) fn split_leading_qualifier(name: &str) -> Option<(&str, &str)> {
    let mut parts = name.splitn(2, "::");
    let qualifier = parts.next()?;
    let member = parts.next()?;
    Some((qualifier, member))
}

pub(crate) fn member_tail(name: &str) -> &str {
    name.rsplit("::").next().unwrap_or(name)
}

pub(crate) fn split_member_tail(name: &str) -> Option<(&str, &str)> {
    let index = name.rfind("::")?;
    Some((&name[..index], &name[index + 2..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_leading_qualifier_uses_first_separator() {
        assert_eq!(
            split_leading_qualifier("Result::Ok"),
            Some(("Result", "Ok"))
        );
        assert_eq!(
            split_leading_qualifier("dep::Result::Ok"),
            Some(("dep", "Result::Ok"))
        );
        assert_eq!(split_leading_qualifier("Ok"), None);
    }

    #[test]
    fn member_tail_uses_last_separator() {
        assert_eq!(member_tail("Result::Ok"), "Ok");
        assert_eq!(member_tail("dep::Result::Ok"), "Ok");
        assert_eq!(member_tail("Ok"), "Ok");
    }

    #[test]
    fn split_member_tail_uses_last_separator() {
        assert_eq!(split_member_tail("Result::Ok"), Some(("Result", "Ok")));
        assert_eq!(
            split_member_tail("dep::Result::Ok"),
            Some(("dep::Result", "Ok"))
        );
        assert_eq!(split_member_tail("Ok"), None);
    }
}
