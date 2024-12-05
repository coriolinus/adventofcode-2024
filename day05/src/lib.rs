use aoclib::CommaSep;
use std::{path::Path, str::FromStr};

type Page = u32;

#[derive(Debug, Clone, Copy, Hash, parse_display::FromStr, parse_display::Display)]
#[display("{prior}|{later}")]
struct PageOrder {
    prior: Page,
    later: Page,
}

impl PageOrder {
    fn checker(&self) -> PageOrderChecker {
        PageOrderChecker {
            order: self,
            first_match: None,
            second_match: None,
        }
    }
}

#[derive(Debug, Clone)]
struct UpdatePages {
    pages: Vec<Page>,
}

impl FromStr for UpdatePages {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pages = s
            .parse::<CommaSep<Page>>()
            .map_err(|_| Error::InvalidInput)?
            .into_iter()
            .collect();
        Ok(Self { pages })
    }
}

impl UpdatePages {
    fn satisfies_rules(&self, rules: &[PageOrder]) -> bool {
        let mut checkers = Vec::with_capacity(rules.len());
        checkers.extend(rules.iter().map(PageOrder::checker));

        for page in self.pages.iter().copied() {
            for checker in checkers.iter_mut() {
                checker.apply(page);
                if checker.outcome() == RuleOutcome::Falsified {
                    return false;
                }
            }
        }

        true
    }

    fn middle_number(&self) -> Page {
        if self.pages.len() % 2 != 1 {
            panic!("even number of pages in print job");
        }

        self.pages[self.pages.len() / 2]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuleOutcome {
    NoPageMatches,
    OnePageMatch,
    Satisfied,
    Falsified,
}

struct PageOrderChecker<'a> {
    order: &'a PageOrder,
    first_match: Option<Page>,
    second_match: Option<Page>,
}

impl<'a> PageOrderChecker<'a> {
    fn outcome(&self) -> RuleOutcome {
        match (self.first_match, self.second_match) {
            (Some(first), Some(second))
                if first == self.order.prior && second == self.order.later =>
            {
                RuleOutcome::Satisfied
            }
            (Some(_), Some(_)) => RuleOutcome::Falsified,
            (Some(_), None) | (None, Some(_)) => RuleOutcome::OnePageMatch,
            (None, None) => RuleOutcome::NoPageMatches,
        }
    }

    fn matches(&self, page: Page) -> bool {
        self.order.prior == page || self.order.later == page
    }

    fn apply(&mut self, page: Page) {
        if !self.matches(page) {
            return;
        }

        if self.first_match.is_none() {
            self.first_match = Some(page);
            return;
        }

        if self.second_match.is_none() {
            self.second_match = Some(page)
        }
    }
}

fn parse(input: &Path) -> Result<(Vec<PageOrder>, Vec<UpdatePages>), Error> {
    let data = std::fs::read_to_string(input)?;
    let (before, after) = data.split_once("\n\n").ok_or(Error::InvalidInput)?;
    let page_orders = before
        .lines()
        .map(PageOrder::from_str)
        .collect::<Result<_, _>>()
        .map_err(|_| Error::InvalidInput)?;
    let update_pages = after
        .lines()
        .map(UpdatePages::from_str)
        .collect::<Result<_, _>>()?;
    Ok((page_orders, update_pages))
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let (page_orders, update_pages) = parse(input)?;

    let middle_page_sum = update_pages
        .iter()
        .filter(|print_job| print_job.satisfies_rules(&page_orders))
        .map(UpdatePages::middle_number)
        .sum::<Page>();
    println!("sum of middle pages: {middle_page_sum}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    unimplemented!("input file: {:?}", input)
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("invalid input")]
    InvalidInput,
    #[error("no solution found")]
    NoSolution,
}
