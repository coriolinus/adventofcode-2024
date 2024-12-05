use aoclib::CommaSep;
use std::{path::Path, str::FromStr};

type Page = u32;

#[derive(Debug, Clone, Copy, Hash, parse_display::FromStr, parse_display::Display)]
#[display("{prior}|{later}")]
struct OrderingRule {
    prior: Page,
    later: Page,
}

impl OrderingRule {
    fn checker(&self) -> PageOrderChecker {
        PageOrderChecker {
            order: self,
            first_match: None,
            second_match: None,
        }
    }
}

#[derive(Debug, Clone)]
struct PrintJob {
    pages: Vec<Page>,
}

impl FromStr for PrintJob {
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

impl PrintJob {
    fn satisfies_rules(&self, rules: &[OrderingRule]) -> bool {
        let mut checkers = Vec::with_capacity(rules.len());
        checkers.extend(rules.iter().map(OrderingRule::checker));

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
    order: &'a OrderingRule,
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

fn parse(input: &Path) -> Result<(Vec<OrderingRule>, Vec<PrintJob>), Error> {
    let data = std::fs::read_to_string(input)?;
    let (before, after) = data.split_once("\n\n").ok_or(Error::InvalidInput)?;
    let ordering_rules = before
        .lines()
        .map(OrderingRule::from_str)
        .collect::<Result<_, _>>()
        .map_err(|_| Error::InvalidInput)?;
    let print_jobs = after
        .lines()
        .map(PrintJob::from_str)
        .collect::<Result<_, _>>()?;
    Ok((ordering_rules, print_jobs))
}

pub fn part1(input: &Path) -> Result<(), Error> {
    let (ordering_rules, print_jobs) = parse(input)?;

    let middle_page_sum = print_jobs
        .iter()
        .filter(|print_job| print_job.satisfies_rules(&ordering_rules))
        .map(PrintJob::middle_number)
        .sum::<Page>();
    println!("sum of middle pages: {middle_page_sum}");
    Ok(())
}

pub fn part2(input: &Path) -> Result<(), Error> {
    let (ordering_rules, mut print_jobs) = parse(input)?;

    // retain only incorrectly ordered jobs
    print_jobs.retain(|job| !job.satisfies_rules(&ordering_rules));
    for job in print_jobs.iter_mut() {
        let relevant_rules =
            {
                let mut rr = Vec::with_capacity(ordering_rules.len());
                rr.extend(ordering_rules.iter().filter(|rule| {
                    job.pages.contains(&rule.prior) && job.pages.contains(&rule.later)
                }));
                rr
            };

        while !job.satisfies_rules(&relevant_rules) {
            for rule in &relevant_rules {
                let p_idx = job
                    .pages
                    .iter()
                    .position(|page| *page == rule.prior)
                    .expect("relevant jobs contain the prior rule");
                let l_idx = job
                    .pages
                    .iter()
                    .position(|page| *page == rule.later)
                    .expect("relevant jobs contain the later rule");
                if p_idx > l_idx {
                    job.pages.swap(p_idx, l_idx);
                }
            }
        }
    }

    let middle_page_sum = print_jobs.iter().map(PrintJob::middle_number).sum::<Page>();

    println!("sum of previously-incorrect middle pages, after reordering: {middle_page_sum}");
    Ok(())
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
