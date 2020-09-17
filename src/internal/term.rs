// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! A term is the fundamental unit of operation of the PubGrub algorithm.
//! It is a positive or negative expression regarding a set of versions.

use crate::range::Range;
use crate::version::Version;

///  A positive or negative expression regarding a set of versions.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Term<V: Clone + Ord + Version> {
    /// For example, "1.0.0 <= v < 2.0.0" is a positive expression
    /// that is evaluated true if a version is selected
    /// and comprised between version 1.0.0 and version 2.0.0.
    Positive(Range<V>),
    /// The term "not v < 3.0.0" is a negative expression
    /// that is evaluated true if a version is selected >= 3.0.0
    /// or if no version is selected at all.
    Negative(Range<V>),
}

// Base methods.
impl<V: Clone + Ord + Version> Term<V> {
    /// Simply check if a term is positive.
    pub fn is_positive(&self) -> bool {
        match self {
            Self::Positive(_) => true,
            Self::Negative(_) => false,
        }
    }

    /// Negate a term.
    /// Evaluation of a negated term always returns
    /// the opposite of the evaluation of the original one.
    pub fn negate(&self) -> Self {
        match self {
            Self::Positive(range) => Self::Negative(range.clone()),
            Self::Negative(range) => Self::Positive(range.clone()),
        }
    }

    /// Evaluate a term regarding a given choice (or absence) of version.
    pub fn accept_optional_version(&self, v_option: &Option<V>) -> bool {
        match (self, v_option) {
            (Self::Negative(_), None) => true,
            (Self::Positive(_), None) => false,
            (_, Some(v)) => self.accept_version(v),
        }
    }

    /// Evaluate a term regarding a given choice of version.
    pub fn accept_version(&self, v: &V) -> bool {
        match self {
            Self::Positive(range) => range.contains(v),
            Self::Negative(range) => !(range.contains(v)),
        }
    }
}

// Set operations with terms.
impl<'a, V: 'a + Clone + Ord + Version> Term<V> {
    /// Compute the intersection of two terms.
    /// If at least one term is positive, the intersection is also positive.
    pub fn intersection(&self, other: &Term<V>) -> Term<V> {
        match (self, other) {
            (Self::Positive(r1), Self::Positive(r2)) => Self::Positive(r1.intersection(r2)),
            (Self::Positive(r1), Self::Negative(r2)) => {
                Self::Positive(r1.intersection(&r2.negate()))
            }
            (Self::Negative(r1), Self::Positive(r2)) => {
                Self::Positive(r1.negate().intersection(r2))
            }
            (Self::Negative(r1), Self::Negative(r2)) => Self::Negative(r1.union(r2)),
        }
    }

    /// Compute the union of two terms.
    /// If at least one term is negative, the union is also negative.
    pub fn union(&self, other: &Term<V>) -> Term<V> {
        (self.negate().intersection(&other.negate())).negate()
    }

    /// Compute the intersection of multiple terms.
    pub fn intersect_all<T: AsRef<Term<V>>>(
        mut all_terms: impl Iterator<Item = T>,
    ) -> Option<Term<V>> {
        all_terms.next().map(|initial_term| {
            all_terms.fold(initial_term.as_ref().clone(), |acc, term| {
                acc.intersection(term.as_ref())
            })
        })
    }

    /// Indicate if this term is a subset of another term.
    /// Just like for sets, we say that t1 is a subset of t2
    /// if and only if t1 ∩ t2 = t1.
    pub fn subset_of(&self, other: &Term<V>) -> bool {
        self == &self.intersection(other)
    }
}

/// Describe a relation between a set of terms S and another term t.
///
/// As a shorthand, we say that a term v
/// satisfies or contradicts a term t if {v} satisfies or contradicts it.
pub enum Relation {
    /// We say that a set of terms S "satisfies" a term t
    /// if t must be true whenever every term in S is true.
    Satisfied,
    /// Conversely, S "contradicts" t if t must be false
    /// whenever every term in S is true.
    Contradicted,
    /// If neither of these is true we say that S is "inconclusive" for t.
    Inconclusive,
}

// Relation between terms.
impl<'a, V: 'a + Clone + Ord + Version> Term<V> {
    /// Check if a set of terms satisfies this term.
    ///
    /// We say that a set of terms S "satisfies" a term t
    /// if t must be true whenever every term in S is true.
    pub fn satisfied_by(&self, terms: impl Iterator<Item = &'a Term<V>>) -> bool {
        match Self::intersect_all(terms) {
            // Negative(Range::none) is always evaluated true.
            None => *self == Self::Negative(Range::none()),
            Some(intersection) => intersection.subset_of(self),
        }
    }

    /// Check if a set of terms contradicts this term.
    ///
    /// We say that a set of terms S "contradicts" a term t
    /// if t must be false whenever every term in S is true.
    pub fn contradicted_by(&self, terms: impl Iterator<Item = &'a Term<V>>) -> bool {
        match Self::intersect_all(terms) {
            // Positive(Range::none) is always evaluated false.
            None => *self == Self::Positive(Range::none()),
            Some(intersection) => intersection.intersection(self) == Self::Positive(Range::none()),
        }
    }

    /// Check if a set of terms satisfies or contradicts a given term.
    /// Otherwise the relation is inconclusive.
    pub fn relation_with<T: AsRef<Term<V>>>(
        &self,
        other_terms: Option<impl Iterator<Item = T>>,
    ) -> Relation {
        let other_terms_intersection = other_terms
            .and_then(|ot| Self::intersect_all(ot))
            .unwrap_or(Self::Negative(Range::none()));
        let full_intersection = self.intersection(&other_terms_intersection);
        if full_intersection == other_terms_intersection {
            Relation::Satisfied
        } else if full_intersection == Self::Positive(Range::none()) {
            Relation::Contradicted
        } else {
            Relation::Inconclusive
        }
    }
}

impl<V: Clone + Ord + Version> AsRef<Term<V>> for Term<V> {
    fn as_ref(&self) -> &Term<V> {
        &self
    }
}