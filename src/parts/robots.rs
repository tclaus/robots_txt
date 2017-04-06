use std::ascii::AsciiExt;

use prelude::*;


#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Robots<'a> {
    pub default_section: Section<'a>,
    pub sections: Vec<Section<'a>>,
}

impl <'a> Render for Robots<'a> {
    fn render_to<W: fmt::Write>(&self, w: &mut W) -> fmt::Result {
        for section in &self.sections {
            section.render_to(w)?;
        }
        self.default_section.render_to(w)?;
        Ok(())
    }
}

impl <'a> Robots<'a> {
    pub fn from_str(input: &'a str) -> Robots<'a> {

        let mut robots = Constructor::default();

        for line in input.lines() {
            let (line, comment) = strip_comment(line);
            match split_kv(line) {
                // Comment line, just skip
                None if comment.is_some() => {},
                // Empty line
                None => robots.end_section(),
                // Some statement
                Some((k, v)) => match k {
                    k if "user-agent".eq_ignore_ascii_case(k) => {
                        if robots.section.has_rules() {
                            robots.end_section();
                        }
                        robots.section.push_ua(v);
                    },
                    k if "disallow".eq_ignore_ascii_case(k) => {
                        robots.section.push_rule(Rule::disallow(v));
                    },
                    k if "allow".eq_ignore_ascii_case(k) => {
                        robots.section.push_rule(Rule::allow(v));
                    },
                    k if "sitemap".eq_ignore_ascii_case(k) => {
                        robots.section.push_sitemap(v).ok();
                    },
                    k if "host".eq_ignore_ascii_case(k) => {
                        robots.section.host = Some(Cow::from(v))
                    },
                    k if "crawl-delay".eq_ignore_ascii_case(k) => {
                        v.parse().map(|v| robots.section.crawl_delay = Some(v)).ok();
                    },
                    k if "request-rate".eq_ignore_ascii_case(k) => {
                        if let Some((r, s)) = split_rr(v) {
                            r.parse().and_then(
                                |r| s.parse().map(
                                    |s| robots.section.req_rate = Some(RequestRate::new(r, s))
                                )
                            ).ok();
                        }
                    },

                    // "Unrecognised headers are ignored"
                    _ => {}
                },
            }
        }

        robots.finalize()
    }
}


struct Constructor<'a> {
    pub default_section: Option<Section<'a>>,
    pub sections: Vec<Section<'a>>,
    pub section: Section<'a>,
}

impl <'a> Default for Constructor<'a> {
    fn default() -> Self {
        Constructor {
            default_section: None,
            sections: Vec::new(),
            section: Section::empty(),
        }
    }
}

impl <'a> Constructor<'a> {
    pub fn end_section(&mut self) {
        if self.section.is_empty() {
            return;
        }
        let section = ::std::mem::replace(&mut self.section, Section::empty());
        match section.is_default() {
            true => match self.default_section {
                Some(ref mut default_section) => default_section.merge(section),
                None => self.default_section = Some(section),
            },
            false => self.sections.push(section),
        }
    }

    pub fn finalize(mut self) -> Robots<'a> {
        self.end_section();
        Robots {
            default_section: self.default_section.unwrap_or_default(),
            sections: self.sections,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render() {
        assert_eq!("User-agent: *\nDisallow:\n\n", Robots::default().render().unwrap());
    }


    static ROBOTS1: &'static str = r#"
# robots.txt for http://www.site.com
User-Agent: *
Disallow: /cyberworld/map/ # this is an infinite virtual URL space
Disallow: /tmp/ # these will soon disappear
"#;

    static RESULT1: &'static str = r#"
User-agent: *
Disallow: /cyberworld/map/
Disallow: /tmp/

"#;


    static ROBOTS2: &'static str = r#"
# robots.txt for http://www.site.com
User-Agent: *
Disallow: /cyberworld/map/ # this is an infinite virtual URL space
# Cybermapper knows where to go
User-Agent: cybermapper
Disallow:
"#;

    static RESULT2: &'static str = r#"
User-agent: cybermapper
Disallow:

User-agent: *
Disallow: /cyberworld/map/

"#;


    static ROBOTS3: &'static str = r#"
# robots.txt for http://www.site.com
User-Agent: *
Disallow: /
"#;

    static RESULT3: &'static str = r#"
User-agent: *
Disallow: /

"#;


    static ROBOTS4: &'static str = r#"
User-Agent: *
Disallow:
Disallow: /private
Crawl-delay: 5
Request-Rate: 3/10
Sitemap: http://example.com/sitemap.xml
Host: example.com
"#;

    static RESULT4: &'static str = r#"
User-agent: *
Disallow:
Disallow: /private
Crawl-delay: 5
Request-rate: 3/10
Sitemap: http://example.com/sitemap.xml
Host: example.com

"#;


    #[test]
    fn parse() {
        let test = |robots, sample| {
            let robots = Robots::from_str(robots);
            assert_eq!((sample as &str).trim_left(), robots.render().unwrap());
        };

        test(ROBOTS1, RESULT1);
        test(ROBOTS2, RESULT2);
        test(ROBOTS3, RESULT3);
        test(ROBOTS4, RESULT4);
    }
}