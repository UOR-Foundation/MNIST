#![forbid(unsafe_code)]
use core::cmp::Ordering;
use core::sync::atomic::{AtomicU64, Ordering as Memory};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Overflow;
pub type R<T> = Result<T, Overflow>;

static WORK: AtomicU64 = AtomicU64::new(0);
pub fn tick(units: u64) { WORK.fetch_add(units, Memory::Relaxed); }
pub fn work() -> u64 { WORK.load(Memory::Relaxed) }

pub fn nat_add(a: u64, b: u64) -> R<u64> { a.checked_add(b).ok_or(Overflow) }
pub fn nat_sub(a: u64, b: u64) -> u64 { a.saturating_sub(b) }
pub fn nat_mul(a: u64, b: u64) -> R<u64> { a.checked_mul(b).ok_or(Overflow) }
pub fn nat_quot(a: u64, b: u64, z: u64) -> u64 { a.checked_div(b).unwrap_or(z) }
pub fn nat_rem(a: u64, b: u64, z: u64) -> u64 { a.checked_rem(b).unwrap_or(z) }
pub fn nat_eq(a: u64, b: u64) -> bool { a == b }
pub fn nat_le(a: u64, b: u64) -> bool { a <= b }
pub fn nat_lt(a: u64, b: u64) -> bool { a < b }
pub fn nat_succ(a: u64) -> R<u64> { a.checked_add(1).ok_or(Overflow) }
pub fn int_add(a: i64, b: i64) -> R<i64> { a.checked_add(b).ok_or(Overflow) }
pub fn int_sub(a: i64, b: i64) -> R<i64> { a.checked_sub(b).ok_or(Overflow) }
pub fn int_mul(a: i64, b: i64) -> R<i64> { a.checked_mul(b).ok_or(Overflow) }
pub fn int_neg(a: i64) -> R<i64> { a.checked_neg().ok_or(Overflow) }
pub fn int_quot(a: i64, b: i64, z: i64) -> R<i64> { if b == 0 { Ok(z) } else { a.checked_div(b).ok_or(Overflow) } }
pub fn int_rem(a: i64, b: i64, z: i64) -> i64 { if b == 0 { z } else { a.wrapping_rem(b) } }
pub fn bool_not(a: bool) -> bool { !a }
pub fn bool_and(a: bool, b: bool) -> bool { a && b }
pub fn bool_or(a: bool, b: bool) -> bool { a || b }

pub trait Same { fn same(&self, other: &Self) -> bool; }
pub trait Key { fn key(&self, other: &Self) -> Ordering; }
macro_rules! scalar {
    ($($t:ty),*) => { $(
        impl Same for $t { fn same(&self, other: &Self) -> bool { self == other } }
        impl Key for $t { fn key(&self, other: &Self) -> Ordering { self.cmp(other) } }
    )* };
}
scalar!(bool, u8, u16, u32, u64, i8, i16, i32, i64);
impl Same for Ordering { fn same(&self, other: &Self) -> bool { self == other } }
impl<A: Key, B: Key> Key for (A, B) {
    fn key(&self, other: &Self) -> Ordering {
        match self.0.key(&other.0) { Ordering::Equal => self.1.key(&other.1), decided => decided }
    }
}
pub fn equal<T: Same>(a: T, b: T) -> bool { a.same(&b) }
pub fn compare<T: Key>(a: T, b: T) -> Ordering { a.key(&b) }

macro_rules! fixed {
    ($m:ident, $t:ident) => {
        pub mod $m {
            pub fn checked_add(a: $t, b: $t) -> Option<$t> { a.checked_add(b) }
            pub fn checked_sub(a: $t, b: $t) -> Option<$t> { a.checked_sub(b) }
            pub fn checked_mul(a: $t, b: $t) -> Option<$t> { a.checked_mul(b) }
            pub fn checked_quot(a: $t, b: $t) -> Option<$t> { a.checked_div(b) }
            pub fn bit_and(a: $t, b: $t) -> $t { a & b }
            pub fn bit_or(a: $t, b: $t) -> $t { a | b }
            pub fn bit_xor(a: $t, b: $t) -> $t { a ^ b }
            pub fn bit_not(a: $t) -> $t { !a }
            pub fn shift_left(a: $t, amount: u32) -> Option<$t> { if amount < $t::BITS { Some(a.wrapping_shl(amount)) } else { None } }
            pub fn shift_right(a: $t, amount: u32) -> Option<$t> { if amount < $t::BITS { Some(a.wrapping_shr(amount)) } else { None } }
            pub fn convert(a: i128) -> Option<$t> { $t::try_from(a).ok() }
        }
    };
}
fixed!(fixed_u8, u8); fixed!(fixed_u16, u16); fixed!(fixed_u32, u32); fixed!(fixed_u64, u64);
fixed!(fixed_i8, i8); fixed!(fixed_i16, i16); fixed!(fixed_i32, i32); fixed!(fixed_i64, i64);
pub fn checked_neg_i8(a: i8) -> Option<i8> { a.checked_neg() }
pub fn checked_neg_i16(a: i16) -> Option<i16> { a.checked_neg() }
pub fn checked_neg_i32(a: i32) -> Option<i32> { a.checked_neg() }
pub fn checked_neg_i64(a: i64) -> Option<i64> { a.checked_neg() }

#[derive(Clone)]
pub struct Str(std::rc::Rc<str>);
impl Str {
    pub fn lit(text: &str) -> Str { Str(std::rc::Rc::from(text)) }
    pub fn text(&self) -> &str { &self.0 }
}
#[derive(Clone)]
pub struct Bytes(std::rc::Rc<[u8]>);
impl Bytes {
    pub fn lit(octets: &[u8]) -> Bytes { Bytes(std::rc::Rc::from(octets)) }
    pub fn octets(&self) -> &[u8] { &self.0 }
}

pub struct Node<T> { head: T, tail: List<T> }
/// A list is a run of cells laid out contiguously, from the newest at the end of a buffer
/// to the oldest at its start, or a cell in front of another list. A `cons` onto a run that
/// owns its buffer alone pushes in place; a `cons` onto any other list is one cell, so
/// building and taking apart a cell is constant work whatever the sharing, and no buffer is
/// written while another list can see it.
pub enum Repr<T> { Nil, Run { buffer: std::rc::Rc<Vec<T>>, len: usize }, Cell(std::rc::Rc<Node<T>>) }
pub struct List<T>(Repr<T>);
impl<T> Clone for List<T> {
    fn clone(&self) -> Self {
        List(match &self.0 {
            Repr::Nil => Repr::Nil,
            Repr::Run { buffer, len } => Repr::Run { buffer: buffer.clone(), len: *len },
            Repr::Cell(cell) => Repr::Cell(cell.clone()),
        })
    }
}
impl<T> List<T> {
    fn take(&mut self) -> Repr<T> { std::mem::replace(&mut self.0, Repr::Nil) }
}
impl<T> Drop for Node<T> {
    fn drop(&mut self) {
        let mut next = match self.tail.take() { Repr::Cell(cell) => Some(cell), _ => None };
        while let Some(cell) = next { // release
            match std::rc::Rc::try_unwrap(cell) {
                Ok(mut node) => next = match node.tail.take() { Repr::Cell(cell) => Some(cell), _ => None },
                Err(_) => break,
            }
        }
    }
}
impl<T: Clone> List<T> {
    pub fn nil() -> Self { List(Repr::Nil) }
    pub fn cons(head: T, mut tail: List<T>) -> Self {
        List(match tail.take() {
            Repr::Nil => {
                let mut buffer = Vec::with_capacity(16);
                buffer.push(head);
                Repr::Run { buffer: std::rc::Rc::new(buffer), len: 1 }
            }
            Repr::Run { mut buffer, len } => {
                if let Some(cells) = std::rc::Rc::get_mut(&mut buffer) {
                    cells.truncate(len);
                    cells.push(head);
                    Repr::Run { buffer, len: len + 1 }
                } else {
                    Repr::Cell(std::rc::Rc::new(Node { head, tail: List(Repr::Run { buffer, len }) }))
                }
            }
            rest => Repr::Cell(std::rc::Rc::new(Node { head, tail: List(rest) })),
        })
    }
    pub fn uncons(&self) -> Option<(T, List<T>)> {
        match &self.0 {
            Repr::Nil => None,
            Repr::Run { buffer, len } => {
                let head = buffer[len - 1].clone();
                Some((head, List(if *len == 1 { Repr::Nil } else { Repr::Run { buffer: buffer.clone(), len: len - 1 } })))
            }
            Repr::Cell(cell) => Some((cell.head.clone(), cell.tail.clone())),
        }
    }
    /// Walk the items in order, ticking one per item, until `each` says to stop: the cells
    /// in front, then the run they end in.
    fn walk(&self, each: &mut dyn FnMut(&T) -> bool) {
        let mut cursor = self;
        while let Repr::Cell(cell) = &cursor.0 { tick(1); if !each(&cell.head) { return; } cursor = &cell.tail; }
        if let Repr::Run { buffer, len } = &cursor.0 {
            for item in buffer[..*len].iter().rev() { tick(1); if !each(item) { return; } }
        }
    }
    fn items(&self) -> Vec<T> {
        let mut out = Vec::new();
        self.walk(&mut |item| { out.push(item.clone()); true });
        out
    }
    fn onto(items: Vec<T>, tail: List<T>) -> List<T> {
        let mut out = tail;
        for item in items.into_iter().rev() { tick(1); out = List::cons(item, out); }
        out
    }
}

fn chars(text: &str) -> u64 {
    let mut count = 0;
    for _ in text.chars() { tick(1); count += 1; }
    count
}

impl Same for Str {
    fn same(&self, other: &Self) -> bool {
        let mut left = self.0.chars();
        let mut right = other.0.chars();
        loop {
            tick(1);
            match (left.next(), right.next()) { (None, None) => return true, (a, b) if a != b => return false, _ => {} }
        }
    }
}
impl Same for Bytes {
    fn same(&self, other: &Self) -> bool { tick(1 + self.0.len().min(other.0.len()) as u64); self.0 == other.0 }
}
impl Key for Str {
    fn key(&self, other: &Self) -> Ordering {
        let mut left = self.0.chars();
        let mut right = other.0.chars();
        loop {
            tick(1);
            match (left.next(), right.next()) {
                (None, None) => return Ordering::Equal,
                (None, Some(_)) => return Ordering::Less,
                (Some(_), None) => return Ordering::Greater,
                (Some(a), Some(b)) => match a.cmp(&b) { Ordering::Equal => {} decided => return decided },
            }
        }
    }
}

pub fn append_list<T: Clone>(a: List<T>, b: List<T>) -> List<T> { List::onto(a.items(), b) }
pub fn append_bytes(a: Bytes, b: Bytes) -> Bytes {
    tick((a.0.len() + b.0.len()) as u64);
    let mut out = Vec::with_capacity(a.0.len() + b.0.len());
    out.extend_from_slice(&a.0);
    out.extend_from_slice(&b.0);
    Bytes(std::rc::Rc::from(out))
}
pub fn length_list<T: Clone>(a: List<T>) -> u64 {
    let mut count = 0;
    a.walk(&mut |_| { count += 1; true });
    count
}
pub fn length_bytes(a: Bytes) -> u64 { a.0.len() as u64 }
pub fn length_string(a: Str) -> u64 { chars(&a.0) }
pub fn index_list<T: Clone>(a: List<T>, i: u64) -> Option<T> {
    let mut position = 0;
    let mut found = None;
    a.walk(&mut |item| { if position == i { found = Some(item.clone()); return false; } position += 1; true });
    found
}
pub fn index_bytes(a: Bytes, i: u64) -> Option<u8> { usize::try_from(i).ok().and_then(|i| a.0.get(i)).copied() }
pub fn slice_list<T: Clone>(a: List<T>, start: u64, count: u64) -> Option<List<T>> {
    let end = u128::from(start) + u128::from(count);
    let mut part = Vec::new();
    let mut position: u128 = 0;
    a.walk(&mut |item| {
        if position >= end { return false; }
        if position >= u128::from(start) { part.push(item.clone()); }
        position += 1;
        true
    });
    if position < end { None } else { Some(List::onto(part, List::nil())) }
}
pub fn slice_bytes(a: Bytes, start: u64, count: u64) -> Option<Bytes> {
    let end = u128::from(start) + u128::from(count);
    if end > a.0.len() as u128 { return None; }
    tick(count);
    Some(Bytes::lit(&a.0[start as usize..end as usize]))
}
pub fn utf8_encode(a: Str) -> Bytes { tick(a.0.len() as u64); Bytes::lit(a.0.as_bytes()) }
pub fn utf8_decode(a: Bytes) -> Option<Str> { tick(a.0.len() as u64); core::str::from_utf8(&a.0).ok().map(Str::lit) }
pub fn compare_bytes(a: Bytes, b: Bytes) -> Ordering { tick(1 + a.0.len().min(b.0.len()) as u64); a.0.cmp(&b.0) }
pub fn split_exact(text: Str, delimiter: Str, maximum: u32) -> Option<List<Str>> {
    if delimiter.0.is_empty() { return None; }
    chars(&text.0);
    let fields: Vec<Str> = text.0.split(&*delimiter.0).map(Str::lit).collect();
    if fields.len() as u128 > u128::from(maximum) { return None; }
    Some(List::onto(fields, List::nil()))
}
pub fn join(texts: List<Str>, delimiter: Str) -> Str {
    let mut out = String::new();
    let mut first = true;
    texts.walk(&mut |text| {
        if !first { chars(&delimiter.0); out.push_str(&delimiter.0); }
        first = false;
        chars(&text.0);
        out.push_str(&text.0);
        true
    });
    Str::lit(&out)
}
fn canonical_decimal(text: &str) -> Option<Option<i128>> {
    let digits = text.strip_prefix('-').unwrap_or(text);
    let mut canonical = !digits.is_empty() && (digits == "0" || !digits.starts_with('0')) && text != "-0";
    for c in digits.chars() { tick(1); canonical = canonical && c.is_ascii_digit(); }
    if !canonical { return None; }
    Some(text.parse::<i128>().ok())
}
pub fn parse_int(text: Str) -> R<Option<i64>> {
    match canonical_decimal(&text.0) {
        None => Ok(None),
        Some(Some(n)) => i64::try_from(n).map(Some).map_err(|_| Overflow),
        Some(None) => Err(Overflow),
    }
}
macro_rules! decimal {
    ($t:ident, $format:ident, $parse:ident) => {
        pub fn $format(a: $t) -> Str { let text = a.to_string(); tick(text.len() as u64); Str::lit(&text) }
        pub fn $parse(text: Str) -> Option<$t> {
            match canonical_decimal(&text.0) { Some(Some(n)) => $t::try_from(n).ok(), _ => None }
        }
    };
}
decimal!(u8, format_u8, parse_u8); decimal!(u16, format_u16, parse_u16);
decimal!(u32, format_u32, parse_u32); decimal!(u64, format_u64, parse_u64);
decimal!(i8, format_i8, parse_i8); decimal!(i16, format_i16, parse_i16);
decimal!(i32, format_i32, parse_i32); decimal!(i64, format_i64, parse_i64);

#[derive(Clone)]
pub enum Fn0 {
    F10(Bytes),
}

impl Fn0 {
    pub fn apply(&self, p0: List<List<i64>>, p1: u64) -> R<List<List<i64>>> {
        match self {
            Fn0::F10(k0) => f10(k0.clone(), p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn1 {
    F13,
}

impl Fn1 {
    pub fn apply(&self, p0: ((i64, u64), u64), p1: i64) -> R<((i64, u64), u64)> {
        match self {
            Fn1::F13 => f13(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn2 {
    F24,
    F85(i64, i64, i64),
    F87,
    F88,
}

impl Fn2 {
    pub fn apply(&self, p0: List<List<i64>>, p1: List<i64>) -> R<List<List<i64>>> {
        match self {
            Fn2::F24 => Ok(f24(p0, p1)),
            Fn2::F85(k0, k1, k2) => f85(*k0, *k1, *k2, p0, p1),
            Fn2::F87 => f87(p0, p1),
            Fn2::F88 => Ok(f88(p0, p1)),
        }
    }
}

#[derive(Clone)]
pub enum Fn3 {
    F33,
}

impl Fn3 {
    pub fn apply(&self, p0: (((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>))), ((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>)))), p1: List<i64>) -> (((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>))), ((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>)))) {
        match self {
            Fn3::F33 => f33(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn4 {
    F43(Bytes, u64),
}

impl Fn4 {
    pub fn apply(&self, p0: List<i64>, p1: u64) -> R<List<i64>> {
        match self {
            Fn4::F43(k0, k1) => f43(k0.clone(), *k1, p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn5 {
    F45,
}

impl Fn5 {
    pub fn apply(&self, p0: List<i64>, p1: List<List<i64>>) -> List<i64> {
        match self {
            Fn5::F45 => f45(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn6 {
    F48,
    F54,
    F55,
    F69,
}

impl Fn6 {
    pub fn apply(&self, p0: List<List<List<i64>>>, p1: List<List<i64>>) -> R<List<List<List<i64>>>> {
        match self {
            Fn6::F48 => f48(p0, p1),
            Fn6::F54 => f54(p0, p1),
            Fn6::F55 => f55(p0, p1),
            Fn6::F69 => Ok(f69(p0, p1)),
        }
    }
}

#[derive(Clone)]
pub enum Fn7 {
    F63,
    F100(i64, i64, i64),
}

impl Fn7 {
    pub fn apply(&self, p0: List<i64>, p1: i64) -> R<List<i64>> {
        match self {
            Fn7::F63 => Ok(f63(p0, p1)),
            Fn7::F100(k0, k1, k2) => f100(*k0, *k1, *k2, p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn8 {
    F66,
}

impl Fn8 {
    pub fn apply(&self, p0: (List<i64>, i64), p1: i64) -> R<(List<i64>, i64)> {
        match self {
            Fn8::F66 => f66(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn9 {
    F82,
}

impl Fn9 {
    pub fn apply(&self, p0: List<i64>, p1: List<i64>) -> List<i64> {
        match self {
            Fn9::F82 => f82(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn10 {
    F84,
    F89,
}

impl Fn10 {
    pub fn apply(&self, p0: ((List<i64>, i64), List<List<i64>>), p1: List<i64>) -> R<((List<i64>, i64), List<List<i64>>)> {
        match self {
            Fn10::F84 => f84(p0, p1),
            Fn10::F89 => f89(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn11 {
    F94(i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64),
}

impl Fn11 {
    pub fn apply(&self, p0: ((List<i64>, (List<i64>, List<i64>)), ((List<i64>, List<List<i64>>), (i64, List<List<i64>>))), p1: List<i64>) -> R<((List<i64>, (List<i64>, List<i64>)), ((List<i64>, List<List<i64>>), (i64, List<List<i64>>)))> {
        match self {
            Fn11::F94(k0, k1, k2, k3, k4, k5, k6, k7, k8, k9, k10, k11, k12, k13, k14, k15, k16, k17, k18, k19, k20, k21, k22, k23, k24) => f94(*k0, *k1, *k2, *k3, *k4, *k5, *k6, *k7, *k8, *k9, *k10, *k11, *k12, *k13, *k14, *k15, *k16, *k17, *k18, *k19, *k20, *k21, *k22, *k23, *k24, p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn12 {
    F97,
}

impl Fn12 {
    pub fn apply(&self, p0: (List<i64>, List<i64>), p1: i64) -> (List<i64>, List<i64>) {
        match self {
            Fn12::F97 => f97(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn13 {
    F99,
    F102,
}

impl Fn13 {
    pub fn apply(&self, p0: ((i64, i64), List<i64>), p1: i64) -> R<((i64, i64), List<i64>)> {
        match self {
            Fn13::F99 => f99(p0, p1),
            Fn13::F102 => f102(p0, p1),
        }
    }
}

#[derive(Clone)]
pub enum Fn14 {
    F104(i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64, i64),
}

impl Fn14 {
    pub fn apply(&self, p0: ((((List<i64>, (List<i64>, List<i64>)), (List<i64>, (List<i64>, i64))), ((i64, (i64, i64)), ((i64, i64), (i64, i64)))), (((i64, (i64, i64)), ((i64, i64), (i64, i64))), ((i64, (i64, i64)), ((i64, i64), (i64, List<i64>))))), p1: i64) -> R<((((List<i64>, (List<i64>, List<i64>)), (List<i64>, (List<i64>, i64))), ((i64, (i64, i64)), ((i64, i64), (i64, i64)))), (((i64, (i64, i64)), ((i64, i64), (i64, i64))), ((i64, (i64, i64)), ((i64, i64), (i64, List<i64>)))))> {
        match self {
            Fn14::F104(k0, k1, k2, k3, k4, k5, k6, k7, k8, k9, k10, k11, k12, k13, k14, k15, k16, k17, k18, k19, k20, k21, k22, k23, k24) => f104(*k0, *k1, *k2, *k3, *k4, *k5, *k6, *k7, *k8, *k9, *k10, *k11, *k12, *k13, *k14, *k15, *k16, *k17, *k18, *k19, *k20, *k21, *k22, *k23, *k24, p0, p1),
        }
    }
}

pub fn f0(v0: Bytes) -> R<Result<u64, (bool, bool)>> {
    let v1: Bytes = v0;
    if f1(v1.clone()) {
        let v2: u64 = f2(v1)?;
        Ok(Ok::<u64, (bool, bool)>(v2))
    } else {
        Ok(Err::<u64, (bool, bool)>((false, false)))
    }
}

pub fn f1(v0: Bytes) -> bool {
    let a2 = {
        let a1 = v0;
        length_bytes(a1)
    };
    let a3 = 784u64;
    nat_eq(a2, a3)
}

pub fn f2(v0: Bytes) -> R<u64> {
    f3(v0)
}

pub fn f3(v0: Bytes) -> R<u64> {
    let a6 = 9u64;
    let a7 = {
        let a4 = 9u64;
        let a5 = f4(f5(v0)?)?;
        nat_sub(a4, a5)
    };
    Ok(nat_sub(a6, a7))
}

pub fn f4(v0: List<List<i64>>) -> R<u64> {
    f6(f7(v0)?)
}

pub fn f5(v0: Bytes) -> R<List<List<i64>>> {
    f8(f9(Fn0::F10(v0), List::<List<i64>>::nil(), f11())?)
}

pub fn f6(v0: List<i64>) -> R<u64> {
    let (_, h8) = {
        let (h9, _) = f12(Fn1::F13, ((-1i64, 0u64), 0u64), v0)?;
        h9
    };
    Ok(h8)
}

pub fn f7(v0: List<List<i64>>) -> R<List<i64>> {
    let v1: List<i64> = f14(v0)?;
    let v2: List<List<i64>> = f15(f16())?;
    let v3: List<i64> = f17();
    let v4: List<i64> = f18();
    Ok(List::cons(f19(v1.clone(), f20(v2.clone(), 0u64), f21(v3.clone(), 0u64), f22(v4.clone(), 0u64)?)?, List::cons(f19(v1.clone(), f20(v2.clone(), 1u64), f21(v3.clone(), 1u64), f22(v4.clone(), 1u64)?)?, List::cons(f19(v1.clone(), f20(v2.clone(), 2u64), f21(v3.clone(), 2u64), f22(v4.clone(), 2u64)?)?, List::cons(f19(v1.clone(), f20(v2.clone(), 3u64), f21(v3.clone(), 3u64), f22(v4.clone(), 3u64)?)?, List::cons(f19(v1.clone(), f20(v2.clone(), 4u64), f21(v3.clone(), 4u64), f22(v4.clone(), 4u64)?)?, List::cons(f19(v1.clone(), f20(v2.clone(), 5u64), f21(v3.clone(), 5u64), f22(v4.clone(), 5u64)?)?, List::cons(f19(v1.clone(), f20(v2.clone(), 6u64), f21(v3.clone(), 6u64), f22(v4.clone(), 6u64)?)?, List::cons(f19(v1.clone(), f20(v2.clone(), 7u64), f21(v3.clone(), 7u64), f22(v4.clone(), 7u64)?)?, List::cons(f19(v1.clone(), f20(v2.clone(), 8u64), f21(v3.clone(), 8u64), f22(v4.clone(), 8u64)?)?, List::cons(f19(v1, f20(v2, 9u64), f21(v3, 9u64), f22(v4, 9u64)?)?, List::<i64>::nil())))))))))))
}

pub fn f8(v0: List<List<i64>>) -> R<List<List<i64>>> {
    f23(Fn2::F24, List::<List<i64>>::nil(), v0)
}

pub fn f9(v0: Fn0, v1: List<List<i64>>, v2: List<u64>) -> R<List<List<i64>>> {
    let m10 = v2;
    match m10.uncons() {
        None => {
            Ok(v1)
        }
        Some((v3, v4)) => {
            f9(v0.clone(), {
                let c11: Fn0 = v0;
                c11.apply(v1, v3)?
            }, v4)
        }
    }
}

pub fn f10(v0: Bytes, v1: List<List<i64>>, v2: u64) -> R<List<List<i64>>> {
    Ok(List::cons(f25(v0, v2)?, v1))
}

pub fn f11() -> List<u64> {
    List::cons(0u64, List::cons(1u64, List::cons(2u64, List::cons(3u64, List::cons(4u64, List::cons(5u64, List::cons(6u64, List::cons(7u64, List::cons(8u64, List::cons(9u64, List::cons(10u64, List::cons(11u64, List::cons(12u64, List::cons(13u64, List::cons(14u64, List::cons(15u64, List::cons(16u64, List::cons(17u64, List::cons(18u64, List::cons(19u64, List::cons(20u64, List::cons(21u64, List::cons(22u64, List::cons(23u64, List::cons(24u64, List::cons(25u64, List::cons(26u64, List::cons(27u64, List::<u64>::nil()))))))))))))))))))))))))))))
}

pub fn f12(v0: Fn1, v1: ((i64, u64), u64), v2: List<i64>) -> R<((i64, u64), u64)> {
    let m12 = v2;
    match m12.uncons() {
        None => {
            Ok(v1)
        }
        Some((v3, v4)) => {
            f12(v0.clone(), {
                let c13: Fn1 = v0;
                c13.apply(v1, v3)?
            }, v4)
        }
    }
}

pub fn f13(v0: ((i64, u64), u64), v1: i64) -> R<((i64, u64), u64)> {
    let v2: i64 = {
        let (h14, _) = {
            let (h15, _) = v0;
            h15
        };
        h14
    };
    let v3: u64 = {
        let (_, h16) = {
            let (h17, _) = v0;
            h17
        };
        h16
    };
    let v4: u64 = {
        let (_, h18) = v0;
        h18
    };
    let v5: bool = {
        let m21 = {
            let a19 = v2;
            let a20 = v1;
            compare(a19, a20)
        };
        match m21 {
            Ordering::Less => {
                true
            }
            Ordering::Equal => {
                false
            }
            Ordering::Greater => {
                false
            }
        }
    };
    Ok(((if v5 {
        v1
    } else {
        v2
    }, if v5 {
        v4
    } else {
        v3
    }), {
        let a22 = v4;
        let a23 = 1u64;
        nat_add(a22, a23)?
    }))
}

pub fn f14(v0: List<List<i64>>) -> R<List<i64>> {
    Ok(f26(f27(f28(f29(f30(f31(v0)?)?)?)?)?))
}

pub fn f15(v0: List<List<i64>>) -> R<List<List<i64>>> {
    let v1: (((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>))), ((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>)))) = f32(Fn3::F33, (((List::<i64>::nil(), List::<i64>::nil()), (List::<i64>::nil(), (List::<i64>::nil(), List::<i64>::nil()))), ((List::<i64>::nil(), List::<i64>::nil()), (List::<i64>::nil(), (List::<i64>::nil(), List::<i64>::nil())))), v0);
    let v2: ((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>))) = {
        let (h24, _) = v1;
        h24
    };
    let v3: (List<i64>, List<i64>) = {
        let (h25, _) = v2;
        h25
    };
    let v4: (List<i64>, (List<i64>, List<i64>)) = {
        let (_, h26) = v2;
        h26
    };
    let v5: (List<i64>, List<i64>) = {
        let (_, h27) = v4;
        h27
    };
    let v6: ((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>))) = {
        let (_, h28) = v1;
        h28
    };
    let v7: (List<i64>, List<i64>) = {
        let (h29, _) = v6;
        h29
    };
    let v8: (List<i64>, (List<i64>, List<i64>)) = {
        let (_, h30) = v6;
        h30
    };
    let v9: (List<i64>, List<i64>) = {
        let (_, h31) = v8;
        h31
    };
    Ok(List::cons(f34({
        let (h32, _) = v3;
        h32
    })?, List::cons(f34({
        let (_, h33) = v3;
        h33
    })?, List::cons(f34({
        let (h34, _) = v4;
        h34
    })?, List::cons(f34({
        let (h35, _) = v5;
        h35
    })?, List::cons(f34({
        let (_, h36) = v5;
        h36
    })?, List::cons(f34({
        let (h37, _) = v7;
        h37
    })?, List::cons(f34({
        let (_, h38) = v7;
        h38
    })?, List::cons(f34({
        let (h39, _) = v8;
        h39
    })?, List::cons(f34({
        let (h40, _) = v9;
        h40
    })?, List::cons(f34({
        let (_, h41) = v9;
        h41
    })?, List::<List<i64>>::nil())))))))))))
}

pub fn f16() -> List<List<i64>> {
    let a54 = {
        let a46 = {
            let a42 = List::cons(List::cons(16i64, List::cons(27i64, List::cons(14i64, List::cons(27i64, List::cons(-16i64, List::cons(-21i64, List::cons(-6i64, List::cons(-16i64, List::cons(-16i64, List::cons(6i64, List::<i64>::nil())))))))))), List::cons(List::cons(-8i64, List::cons(30i64, List::cons(24i64, List::cons(-26i64, List::cons(34i64, List::cons(-42i64, List::cons(73i64, List::cons(10i64, List::cons(-18i64, List::cons(-62i64, List::<i64>::nil())))))))))), List::cons(List::cons(-33i64, List::cons(21i64, List::cons(15i64, List::cons(-2i64, List::cons(38i64, List::cons(-84i64, List::cons(0i64, List::cons(-2i64, List::cons(3i64, List::cons(-51i64, List::<i64>::nil())))))))))), List::cons(List::cons(3i64, List::cons(5i64, List::cons(47i64, List::cons(37i64, List::cons(-12i64, List::cons(3i64, List::cons(-18i64, List::cons(-14i64, List::cons(6i64, List::cons(0i64, List::<i64>::nil())))))))))), List::cons(List::cons(-26i64, List::cons(26i64, List::cons(18i64, List::cons(21i64, List::cons(-11i64, List::cons(-5i64, List::cons(19i64, List::cons(-14i64, List::cons(9i64, List::cons(-29i64, List::<i64>::nil())))))))))), List::cons(List::cons(-29i64, List::cons(23i64, List::cons(-29i64, List::cons(-18i64, List::cons(76i64, List::cons(6i64, List::cons(-28i64, List::cons(34i64, List::cons(-11i64, List::cons(-83i64, List::<i64>::nil())))))))))), List::cons(List::cons(56i64, List::cons(39i64, List::cons(68i64, List::cons(27i64, List::cons(-45i64, List::cons(-87i64, List::cons(-64i64, List::cons(-4i64, List::cons(-14i64, List::cons(8i64, List::<i64>::nil())))))))))), List::cons(List::cons(65i64, List::cons(-51i64, List::cons(67i64, List::cons(5i64, List::cons(13i64, List::cons(-125i64, List::cons(-11i64, List::cons(8i64, List::cons(-4i64, List::cons(24i64, List::<i64>::nil())))))))))), List::cons(List::cons(-22i64, List::cons(6i64, List::cons(-1i64, List::cons(39i64, List::cons(-5i64, List::cons(-5i64, List::cons(23i64, List::cons(-8i64, List::cons(6i64, List::cons(4i64, List::<i64>::nil())))))))))), List::cons(List::cons(-1i64, List::cons(-11i64, List::cons(-26i64, List::cons(24i64, List::cons(-28i64, List::cons(65i64, List::cons(-30i64, List::cons(28i64, List::cons(-15i64, List::cons(-29i64, List::<i64>::nil())))))))))), List::cons(List::cons(-6i64, List::cons(9i64, List::cons(32i64, List::cons(9i64, List::cons(-27i64, List::cons(82i64, List::cons(13i64, List::cons(-74i64, List::cons(5i64, List::cons(-39i64, List::<i64>::nil())))))))))), List::cons(List::cons(-17i64, List::cons(-89i64, List::cons(-62i64, List::cons(53i64, List::cons(-86i64, List::cons(106i64, List::cons(96i64, List::cons(-18i64, List::cons(19i64, List::cons(-19i64, List::<i64>::nil())))))))))), List::cons(List::cons(-7i64, List::cons(-25i64, List::cons(0i64, List::cons(-1i64, List::cons(-17i64, List::cons(4i64, List::cons(-12i64, List::cons(-11i64, List::cons(9i64, List::cons(10i64, List::<i64>::nil())))))))))), List::cons(List::cons(31i64, List::cons(24i64, List::cons(-32i64, List::cons(2i64, List::cons(10i64, List::cons(3i64, List::cons(-12i64, List::cons(18i64, List::cons(-26i64, List::cons(3i64, List::<i64>::nil())))))))))), List::cons(List::cons(-57i64, List::cons(-7i64, List::cons(-26i64, List::cons(127i64, List::cons(-12i64, List::cons(96i64, List::cons(28i64, List::cons(-34i64, List::cons(9i64, List::cons(-45i64, List::<i64>::nil())))))))))), List::cons(List::cons(-82i64, List::cons(9i64, List::cons(-4i64, List::cons(65i64, List::cons(-36i64, List::cons(15i64, List::cons(46i64, List::cons(-49i64, List::cons(29i64, List::cons(-34i64, List::<i64>::nil())))))))))), List::cons(List::cons(-21i64, List::cons(1i64, List::cons(37i64, List::cons(30i64, List::cons(-5i64, List::cons(-52i64, List::cons(4i64, List::cons(11i64, List::cons(-16i64, List::cons(17i64, List::<i64>::nil())))))))))), List::cons(List::cons(-23i64, List::cons(12i64, List::cons(-13i64, List::cons(2i64, List::cons(-34i64, List::cons(21i64, List::cons(-31i64, List::cons(61i64, List::cons(-25i64, List::cons(1i64, List::<i64>::nil())))))))))), List::cons(List::cons(-36i64, List::cons(-22i64, List::cons(15i64, List::cons(59i64, List::cons(-24i64, List::cons(16i64, List::cons(-12i64, List::cons(23i64, List::cons(13i64, List::cons(-1i64, List::<i64>::nil())))))))))), List::cons(List::cons(-7i64, List::cons(19i64, List::cons(8i64, List::cons(-8i64, List::cons(13i64, List::cons(16i64, List::cons(23i64, List::cons(-11i64, List::cons(1i64, List::cons(13i64, List::<i64>::nil())))))))))), List::cons(List::cons(-4i64, List::cons(-16i64, List::cons(40i64, List::cons(44i64, List::cons(-21i64, List::cons(-70i64, List::cons(-45i64, List::cons(29i64, List::cons(-8i64, List::cons(0i64, List::<i64>::nil())))))))))), List::cons(List::cons(-6i64, List::cons(55i64, List::cons(49i64, List::cons(38i64, List::cons(-67i64, List::cons(-33i64, List::cons(-84i64, List::cons(127i64, List::cons(-5i64, List::cons(-7i64, List::<i64>::nil())))))))))), List::cons(List::cons(10i64, List::cons(-62i64, List::cons(11i64, List::cons(-60i64, List::cons(-49i64, List::cons(59i64, List::cons(-17i64, List::cons(61i64, List::cons(-15i64, List::cons(-10i64, List::<i64>::nil())))))))))), List::cons(List::cons(15i64, List::cons(-27i64, List::cons(-79i64, List::cons(-26i64, List::cons(-12i64, List::cons(94i64, List::cons(28i64, List::cons(6i64, List::cons(-1i64, List::cons(18i64, List::<i64>::nil())))))))))), List::cons(List::cons(-31i64, List::cons(-46i64, List::cons(28i64, List::cons(45i64, List::cons(17i64, List::cons(31i64, List::cons(-9i64, List::cons(51i64, List::cons(-13i64, List::cons(-30i64, List::<i64>::nil())))))))))), List::cons(List::cons(-24i64, List::cons(-1i64, List::cons(-26i64, List::cons(-13i64, List::cons(30i64, List::cons(51i64, List::cons(-16i64, List::cons(-20i64, List::cons(3i64, List::cons(56i64, List::<i64>::nil())))))))))), List::cons(List::cons(-25i64, List::cons(-63i64, List::cons(-42i64, List::cons(-21i64, List::cons(95i64, List::cons(-54i64, List::cons(32i64, List::cons(-40i64, List::cons(19i64, List::cons(-2i64, List::<i64>::nil())))))))))), List::cons(List::cons(-6i64, List::cons(-20i64, List::cons(107i64, List::cons(36i64, List::cons(21i64, List::cons(-18i64, List::cons(37i64, List::cons(45i64, List::cons(6i64, List::cons(-127i64, List::<i64>::nil())))))))))), List::cons(List::cons(-34i64, List::cons(76i64, List::cons(-23i64, List::cons(62i64, List::cons(35i64, List::cons(18i64, List::cons(-42i64, List::cons(27i64, List::cons(-24i64, List::cons(3i64, List::<i64>::nil())))))))))), List::cons(List::cons(2i64, List::cons(38i64, List::cons(8i64, List::cons(17i64, List::cons(-8i64, List::cons(5i64, List::cons(-9i64, List::cons(-83i64, List::cons(-7i64, List::cons(17i64, List::<i64>::nil())))))))))), List::cons(List::cons(46i64, List::cons(-9i64, List::cons(16i64, List::cons(-9i64, List::cons(-13i64, List::cons(0i64, List::cons(6i64, List::cons(-53i64, List::cons(10i64, List::cons(-48i64, List::<i64>::nil())))))))))), List::cons(List::cons(-13i64, List::cons(55i64, List::cons(108i64, List::cons(-63i64, List::cons(57i64, List::cons(-80i64, List::cons(-11i64, List::cons(7i64, List::cons(-10i64, List::cons(-24i64, List::<i64>::nil())))))))))), List::<List<i64>>::nil()))))))))))))))))))))))))))))))));
            let a43 = List::cons(List::cons(6i64, List::cons(-29i64, List::cons(2i64, List::cons(-9i64, List::cons(-4i64, List::cons(-33i64, List::cons(8i64, List::cons(1i64, List::cons(-4i64, List::cons(-20i64, List::<i64>::nil())))))))))), List::cons(List::cons(7i64, List::cons(-43i64, List::cons(25i64, List::cons(14i64, List::cons(4i64, List::cons(-34i64, List::cons(-35i64, List::cons(42i64, List::cons(15i64, List::cons(-2i64, List::<i64>::nil())))))))))), List::cons(List::cons(21i64, List::cons(-4i64, List::cons(-2i64, List::cons(25i64, List::cons(-55i64, List::cons(-24i64, List::cons(-25i64, List::cons(37i64, List::cons(31i64, List::cons(42i64, List::<i64>::nil())))))))))), List::cons(List::cons(30i64, List::cons(22i64, List::cons(-42i64, List::cons(-24i64, List::cons(-10i64, List::cons(7i64, List::cons(-7i64, List::cons(-15i64, List::cons(-2i64, List::cons(6i64, List::<i64>::nil())))))))))), List::cons(List::cons(-18i64, List::cons(15i64, List::cons(46i64, List::cons(14i64, List::cons(-27i64, List::cons(12i64, List::cons(-28i64, List::cons(70i64, List::cons(7i64, List::cons(-11i64, List::<i64>::nil())))))))))), List::cons(List::cons(-48i64, List::cons(40i64, List::cons(82i64, List::cons(65i64, List::cons(-45i64, List::cons(-1i64, List::cons(-86i64, List::cons(58i64, List::cons(10i64, List::cons(-46i64, List::<i64>::nil())))))))))), List::cons(List::cons(33i64, List::cons(-53i64, List::cons(37i64, List::cons(39i64, List::cons(-56i64, List::cons(5i64, List::cons(-21i64, List::cons(57i64, List::cons(-25i64, List::cons(5i64, List::<i64>::nil())))))))))), List::cons(List::cons(4i64, List::cons(-13i64, List::cons(-2i64, List::cons(-55i64, List::cons(-55i64, List::cons(93i64, List::cons(70i64, List::cons(4i64, List::cons(-24i64, List::cons(0i64, List::<i64>::nil())))))))))), List::cons(List::cons(-54i64, List::cons(7i64, List::cons(29i64, List::cons(47i64, List::cons(44i64, List::cons(20i64, List::cons(-49i64, List::cons(56i64, List::cons(30i64, List::cons(-63i64, List::<i64>::nil())))))))))), List::cons(List::cons(-115i64, List::cons(-93i64, List::cons(-29i64, List::cons(48i64, List::cons(86i64, List::cons(93i64, List::cons(-127i64, List::cons(-12i64, List::cons(-3i64, List::cons(103i64, List::<i64>::nil())))))))))), List::cons(List::cons(47i64, List::cons(-40i64, List::cons(2i64, List::cons(60i64, List::cons(8i64, List::cons(36i64, List::cons(-25i64, List::cons(16i64, List::cons(-15i64, List::cons(-8i64, List::<i64>::nil())))))))))), List::cons(List::cons(23i64, List::cons(25i64, List::cons(0i64, List::cons(3i64, List::cons(-21i64, List::cons(39i64, List::cons(17i64, List::cons(-1i64, List::cons(-10i64, List::cons(-31i64, List::<i64>::nil())))))))))), List::cons(List::cons(16i64, List::cons(48i64, List::cons(2i64, List::cons(4i64, List::cons(16i64, List::cons(11i64, List::cons(15i64, List::cons(-44i64, List::cons(5i64, List::cons(5i64, List::<i64>::nil())))))))))), List::cons(List::cons(13i64, List::cons(-47i64, List::cons(-31i64, List::cons(59i64, List::cons(-38i64, List::cons(45i64, List::cons(37i64, List::cons(-91i64, List::cons(-8i64, List::cons(35i64, List::<i64>::nil())))))))))), List::cons(List::cons(29i64, List::cons(118i64, List::cons(-33i64, List::cons(-1i64, List::cons(-32i64, List::cons(-23i64, List::cons(12i64, List::cons(-21i64, List::cons(6i64, List::cons(-50i64, List::<i64>::nil())))))))))), List::cons(List::cons(0i64, List::cons(38i64, List::cons(127i64, List::cons(-59i64, List::cons(10i64, List::cons(-27i64, List::cons(-4i64, List::cons(-4i64, List::cons(-25i64, List::cons(-46i64, List::<i64>::nil())))))))))), List::cons(List::cons(17i64, List::cons(-29i64, List::cons(-22i64, List::cons(33i64, List::cons(22i64, List::cons(-16i64, List::cons(25i64, List::cons(19i64, List::cons(-12i64, List::cons(1i64, List::<i64>::nil())))))))))), List::cons(List::cons(14i64, List::cons(-13i64, List::cons(-5i64, List::cons(36i64, List::cons(-19i64, List::cons(24i64, List::cons(20i64, List::cons(14i64, List::cons(-15i64, List::cons(-1i64, List::<i64>::nil())))))))))), List::cons(List::cons(-13i64, List::cons(17i64, List::cons(24i64, List::cons(-16i64, List::cons(-27i64, List::cons(-9i64, List::cons(20i64, List::cons(-28i64, List::cons(-11i64, List::cons(-29i64, List::<i64>::nil())))))))))), List::cons(List::cons(17i64, List::cons(11i64, List::cons(6i64, List::cons(0i64, List::cons(-18i64, List::cons(15i64, List::cons(-2i64, List::cons(8i64, List::cons(-10i64, List::cons(-11i64, List::<i64>::nil())))))))))), List::cons(List::cons(3i64, List::cons(23i64, List::cons(19i64, List::cons(38i64, List::cons(-11i64, List::cons(6i64, List::cons(11i64, List::cons(-18i64, List::cons(-5i64, List::cons(-15i64, List::<i64>::nil())))))))))), List::cons(List::cons(15i64, List::cons(-15i64, List::cons(54i64, List::cons(-30i64, List::cons(-10i64, List::cons(32i64, List::cons(-10i64, List::cons(-24i64, List::cons(-30i64, List::cons(-1i64, List::<i64>::nil())))))))))), List::cons(List::cons(-29i64, List::cons(88i64, List::cons(17i64, List::cons(-7i64, List::cons(-52i64, List::cons(-1i64, List::cons(-3i64, List::cons(-4i64, List::cons(-22i64, List::cons(19i64, List::<i64>::nil())))))))))), List::cons(List::cons(-8i64, List::cons(27i64, List::cons(19i64, List::cons(78i64, List::cons(-31i64, List::cons(-25i64, List::cons(37i64, List::cons(-12i64, List::cons(-6i64, List::cons(-2i64, List::<i64>::nil())))))))))), List::cons(List::cons(17i64, List::cons(-24i64, List::cons(5i64, List::cons(-21i64, List::cons(4i64, List::cons(-45i64, List::cons(5i64, List::cons(0i64, List::cons(13i64, List::cons(16i64, List::<i64>::nil())))))))))), List::cons(List::cons(54i64, List::cons(-36i64, List::cons(27i64, List::cons(-54i64, List::cons(5i64, List::cons(18i64, List::cons(11i64, List::cons(67i64, List::cons(-43i64, List::cons(-32i64, List::<i64>::nil())))))))))), List::cons(List::cons(-41i64, List::cons(82i64, List::cons(-58i64, List::cons(1i64, List::cons(-7i64, List::cons(11i64, List::cons(-45i64, List::cons(-10i64, List::cons(-38i64, List::cons(35i64, List::<i64>::nil())))))))))), List::cons(List::cons(46i64, List::cons(47i64, List::cons(-59i64, List::cons(-2i64, List::cons(-66i64, List::cons(3i64, List::cons(5i64, List::cons(24i64, List::cons(-22i64, List::cons(47i64, List::<i64>::nil())))))))))), List::cons(List::cons(0i64, List::cons(-42i64, List::cons(26i64, List::cons(-5i64, List::cons(5i64, List::cons(22i64, List::cons(-3i64, List::cons(-10i64, List::cons(-9i64, List::cons(-12i64, List::<i64>::nil())))))))))), List::cons(List::cons(-18i64, List::cons(9i64, List::cons(57i64, List::cons(-77i64, List::cons(11i64, List::cons(-17i64, List::cons(20i64, List::cons(36i64, List::cons(31i64, List::cons(-60i64, List::<i64>::nil())))))))))), List::cons(List::cons(-53i64, List::cons(19i64, List::cons(-67i64, List::cons(11i64, List::cons(43i64, List::cons(-8i64, List::cons(-15i64, List::cons(66i64, List::cons(27i64, List::cons(5i64, List::<i64>::nil())))))))))), List::cons(List::cons(-33i64, List::cons(2i64, List::cons(-27i64, List::cons(22i64, List::cons(12i64, List::cons(13i64, List::cons(-23i64, List::cons(-33i64, List::cons(0i64, List::cons(2i64, List::<i64>::nil())))))))))), List::<List<i64>>::nil()))))))))))))))))))))))))))))))));
            append_list(a42, a43)
        };
        let a47 = {
            let a44 = List::cons(List::cons(-3i64, List::cons(1i64, List::cons(-2i64, List::cons(25i64, List::cons(11i64, List::cons(-19i64, List::cons(20i64, List::cons(3i64, List::cons(3i64, List::cons(-5i64, List::<i64>::nil())))))))))), List::cons(List::cons(15i64, List::cons(37i64, List::cons(-18i64, List::cons(-25i64, List::cons(-23i64, List::cons(-2i64, List::cons(18i64, List::cons(-12i64, List::cons(5i64, List::cons(14i64, List::<i64>::nil())))))))))), List::cons(List::cons(3i64, List::cons(57i64, List::cons(-18i64, List::cons(-31i64, List::cons(14i64, List::cons(43i64, List::cons(-7i64, List::cons(-13i64, List::cons(-34i64, List::cons(-4i64, List::<i64>::nil())))))))))), List::cons(List::cons(-25i64, List::cons(-16i64, List::cons(-26i64, List::cons(-34i64, List::cons(-11i64, List::cons(66i64, List::cons(1i64, List::cons(6i64, List::cons(-30i64, List::cons(3i64, List::<i64>::nil())))))))))), List::cons(List::cons(4i64, List::cons(15i64, List::cons(33i64, List::cons(21i64, List::cons(-19i64, List::cons(22i64, List::cons(-3i64, List::cons(9i64, List::cons(12i64, List::cons(-7i64, List::<i64>::nil())))))))))), List::cons(List::cons(-5i64, List::cons(49i64, List::cons(21i64, List::cons(-11i64, List::cons(20i64, List::cons(16i64, List::cons(9i64, List::cons(33i64, List::cons(-35i64, List::cons(-34i64, List::<i64>::nil())))))))))), List::cons(List::cons(-40i64, List::cons(-61i64, List::cons(10i64, List::cons(42i64, List::cons(13i64, List::cons(-28i64, List::cons(-35i64, List::cons(-13i64, List::cons(-2i64, List::cons(104i64, List::<i64>::nil())))))))))), List::cons(List::cons(-22i64, List::cons(-20i64, List::cons(-17i64, List::cons(63i64, List::cons(-9i64, List::cons(11i64, List::cons(-14i64, List::cons(3i64, List::cons(36i64, List::cons(-7i64, List::<i64>::nil())))))))))), List::cons(List::cons(-30i64, List::cons(29i64, List::cons(-2i64, List::cons(-26i64, List::cons(11i64, List::cons(-8i64, List::cons(25i64, List::cons(11i64, List::cons(-21i64, List::cons(-19i64, List::<i64>::nil())))))))))), List::cons(List::cons(-2i64, List::cons(24i64, List::cons(50i64, List::cons(-27i64, List::cons(-6i64, List::cons(55i64, List::cons(-45i64, List::cons(-9i64, List::cons(-9i64, List::cons(20i64, List::<i64>::nil())))))))))), List::cons(List::cons(-1i64, List::cons(-65i64, List::cons(13i64, List::cons(84i64, List::cons(14i64, List::cons(12i64, List::cons(25i64, List::cons(21i64, List::cons(-6i64, List::cons(42i64, List::<i64>::nil())))))))))), List::cons(List::cons(0i64, List::cons(-40i64, List::cons(-27i64, List::cons(-26i64, List::cons(-19i64, List::cons(-20i64, List::cons(29i64, List::cons(15i64, List::cons(12i64, List::cons(-32i64, List::<i64>::nil())))))))))), List::cons(List::cons(-26i64, List::cons(12i64, List::cons(22i64, List::cons(-7i64, List::cons(-2i64, List::cons(15i64, List::cons(11i64, List::cons(-10i64, List::cons(-16i64, List::cons(22i64, List::<i64>::nil())))))))))), List::cons(List::cons(41i64, List::cons(-37i64, List::cons(15i64, List::cons(8i64, List::cons(-15i64, List::cons(-1i64, List::cons(-23i64, List::cons(-39i64, List::cons(4i64, List::cons(-28i64, List::<i64>::nil())))))))))), List::cons(List::cons(30i64, List::cons(-21i64, List::cons(-3i64, List::cons(30i64, List::cons(-53i64, List::cons(40i64, List::cons(57i64, List::cons(-81i64, List::cons(48i64, List::cons(-51i64, List::<i64>::nil())))))))))), List::cons(List::cons(-26i64, List::cons(40i64, List::cons(-15i64, List::cons(36i64, List::cons(7i64, List::cons(34i64, List::cons(0i64, List::cons(-52i64, List::cons(16i64, List::cons(-11i64, List::<i64>::nil())))))))))), List::cons(List::cons(-28i64, List::cons(-30i64, List::cons(14i64, List::cons(-9i64, List::cons(-6i64, List::cons(-34i64, List::cons(7i64, List::cons(21i64, List::cons(-5i64, List::cons(-11i64, List::<i64>::nil())))))))))), List::cons(List::cons(-23i64, List::cons(20i64, List::cons(24i64, List::cons(43i64, List::cons(-5i64, List::cons(9i64, List::cons(11i64, List::cons(-8i64, List::cons(10i64, List::cons(7i64, List::<i64>::nil())))))))))), List::cons(List::cons(-9i64, List::cons(-34i64, List::cons(32i64, List::cons(29i64, List::cons(-2i64, List::cons(12i64, List::cons(-5i64, List::cons(3i64, List::cons(-8i64, List::cons(7i64, List::<i64>::nil())))))))))), List::cons(List::cons(-6i64, List::cons(-15i64, List::cons(-8i64, List::cons(-32i64, List::cons(8i64, List::cons(-3i64, List::cons(19i64, List::cons(17i64, List::cons(-15i64, List::cons(-7i64, List::<i64>::nil())))))))))), List::cons(List::cons(-15i64, List::cons(52i64, List::cons(-43i64, List::cons(-10i64, List::cons(5i64, List::cons(10i64, List::cons(1i64, List::cons(27i64, List::cons(11i64, List::cons(-35i64, List::<i64>::nil())))))))))), List::cons(List::cons(-16i64, List::cons(56i64, List::cons(-25i64, List::cons(-82i64, List::cons(25i64, List::cons(19i64, List::cons(-38i64, List::cons(0i64, List::cons(-12i64, List::cons(15i64, List::<i64>::nil())))))))))), List::cons(List::cons(90i64, List::cons(41i64, List::cons(25i64, List::cons(-17i64, List::cons(12i64, List::cons(45i64, List::cons(-78i64, List::cons(0i64, List::cons(-43i64, List::cons(-22i64, List::<i64>::nil())))))))))), List::cons(List::cons(-4i64, List::cons(15i64, List::cons(-7i64, List::cons(14i64, List::cons(-10i64, List::cons(9i64, List::cons(-2i64, List::cons(-20i64, List::cons(-25i64, List::cons(-46i64, List::<i64>::nil())))))))))), List::cons(List::cons(-2i64, List::cons(-64i64, List::cons(-66i64, List::cons(-80i64, List::cons(55i64, List::cons(-9i64, List::cons(-7i64, List::cons(-44i64, List::cons(32i64, List::cons(73i64, List::<i64>::nil())))))))))), List::cons(List::cons(10i64, List::cons(-2i64, List::cons(-63i64, List::cons(-48i64, List::cons(41i64, List::cons(32i64, List::cons(-30i64, List::cons(-26i64, List::cons(19i64, List::cons(60i64, List::<i64>::nil())))))))))), List::cons(List::cons(-7i64, List::cons(82i64, List::cons(-75i64, List::cons(41i64, List::cons(-2i64, List::cons(-27i64, List::cons(-100i64, List::cons(-22i64, List::cons(19i64, List::cons(64i64, List::<i64>::nil())))))))))), List::cons(List::cons(26i64, List::cons(-35i64, List::cons(6i64, List::cons(-4i64, List::cons(-6i64, List::cons(-33i64, List::cons(19i64, List::cons(13i64, List::cons(16i64, List::cons(-7i64, List::<i64>::nil())))))))))), List::cons(List::cons(12i64, List::cons(-34i64, List::cons(27i64, List::cons(50i64, List::cons(8i64, List::cons(-6i64, List::cons(18i64, List::cons(-32i64, List::cons(15i64, List::cons(-15i64, List::<i64>::nil())))))))))), List::cons(List::cons(45i64, List::cons(-62i64, List::cons(107i64, List::cons(-30i64, List::cons(11i64, List::cons(6i64, List::cons(48i64, List::cons(-38i64, List::cons(17i64, List::cons(-40i64, List::<i64>::nil())))))))))), List::cons(List::cons(-38i64, List::cons(-3i64, List::cons(75i64, List::cons(-3i64, List::cons(-32i64, List::cons(-51i64, List::cons(31i64, List::cons(12i64, List::cons(-13i64, List::cons(-12i64, List::<i64>::nil())))))))))), List::cons(List::cons(19i64, List::cons(-38i64, List::cons(-15i64, List::cons(37i64, List::cons(-10i64, List::cons(21i64, List::cons(29i64, List::cons(-60i64, List::cons(-4i64, List::cons(-20i64, List::<i64>::nil())))))))))), List::<List<i64>>::nil()))))))))))))))))))))))))))))))));
            let a45 = List::cons(List::cons(-23i64, List::cons(-35i64, List::cons(44i64, List::cons(24i64, List::cons(-15i64, List::cons(-32i64, List::cons(-9i64, List::cons(8i64, List::cons(9i64, List::cons(14i64, List::<i64>::nil())))))))))), List::cons(List::cons(14i64, List::cons(-53i64, List::cons(61i64, List::cons(-22i64, List::cons(-26i64, List::cons(9i64, List::cons(35i64, List::cons(-42i64, List::cons(14i64, List::cons(0i64, List::<i64>::nil())))))))))), List::cons(List::cons(-5i64, List::cons(27i64, List::cons(-21i64, List::cons(-41i64, List::cons(27i64, List::cons(6i64, List::cons(27i64, List::cons(-9i64, List::cons(3i64, List::cons(-38i64, List::<i64>::nil())))))))))), List::cons(List::cons(-23i64, List::cons(67i64, List::cons(-15i64, List::cons(-40i64, List::cons(66i64, List::cons(-41i64, List::cons(30i64, List::cons(-5i64, List::cons(12i64, List::cons(-15i64, List::<i64>::nil())))))))))), List::cons(List::cons(-1i64, List::cons(33i64, List::cons(9i64, List::cons(37i64, List::cons(3i64, List::cons(-51i64, List::cons(-39i64, List::cons(-25i64, List::cons(-5i64, List::cons(43i64, List::<i64>::nil())))))))))), List::cons(List::cons(30i64, List::cons(-91i64, List::cons(4i64, List::cons(-38i64, List::cons(-84i64, List::cons(23i64, List::cons(-53i64, List::cons(6i64, List::cons(21i64, List::cons(79i64, List::<i64>::nil())))))))))), List::cons(List::cons(-18i64, List::cons(1i64, List::cons(-8i64, List::cons(-106i64, List::cons(7i64, List::cons(16i64, List::cons(89i64, List::cons(-11i64, List::cons(16i64, List::cons(-7i64, List::<i64>::nil())))))))))), List::cons(List::cons(-37i64, List::cons(54i64, List::cons(-22i64, List::cons(23i64, List::cons(118i64, List::cons(-24i64, List::cons(-46i64, List::cons(21i64, List::cons(0i64, List::cons(-104i64, List::<i64>::nil())))))))))), List::cons(List::cons(11i64, List::cons(6i64, List::cons(55i64, List::cons(-6i64, List::cons(18i64, List::cons(21i64, List::cons(-35i64, List::cons(41i64, List::cons(12i64, List::cons(-54i64, List::<i64>::nil())))))))))), List::cons(List::cons(-24i64, List::cons(-14i64, List::cons(33i64, List::cons(7i64, List::cons(-2i64, List::cons(4i64, List::cons(6i64, List::cons(17i64, List::cons(1i64, List::cons(-71i64, List::<i64>::nil())))))))))), List::cons(List::cons(22i64, List::cons(76i64, List::cons(-47i64, List::cons(58i64, List::cons(34i64, List::cons(-36i64, List::cons(57i64, List::cons(-52i64, List::cons(0i64, List::cons(-21i64, List::<i64>::nil())))))))))), List::cons(List::cons(40i64, List::cons(54i64, List::cons(33i64, List::cons(-24i64, List::cons(-42i64, List::cons(-21i64, List::cons(-57i64, List::cons(-36i64, List::cons(-8i64, List::cons(64i64, List::<i64>::nil())))))))))), List::cons(List::cons(-40i64, List::cons(91i64, List::cons(-20i64, List::cons(9i64, List::cons(10i64, List::cons(80i64, List::cons(-58i64, List::cons(-3i64, List::cons(-32i64, List::cons(-15i64, List::<i64>::nil())))))))))), List::cons(List::cons(-18i64, List::cons(1i64, List::cons(-21i64, List::cons(-8i64, List::cons(-67i64, List::cons(4i64, List::cons(-31i64, List::cons(55i64, List::cons(-27i64, List::cons(19i64, List::<i64>::nil())))))))))), List::cons(List::cons(17i64, List::cons(28i64, List::cons(1i64, List::cons(28i64, List::cons(-17i64, List::cons(43i64, List::cons(-54i64, List::cons(25i64, List::cons(6i64, List::cons(30i64, List::<i64>::nil())))))))))), List::cons(List::cons(-13i64, List::cons(4i64, List::cons(-40i64, List::cons(66i64, List::cons(-70i64, List::cons(51i64, List::cons(7i64, List::cons(3i64, List::cons(34i64, List::cons(-4i64, List::<i64>::nil())))))))))), List::cons(List::cons(8i64, List::cons(-3i64, List::cons(11i64, List::cons(1i64, List::cons(-30i64, List::cons(17i64, List::cons(5i64, List::cons(-6i64, List::cons(13i64, List::cons(10i64, List::<i64>::nil())))))))))), List::cons(List::cons(19i64, List::cons(-20i64, List::cons(28i64, List::cons(-70i64, List::cons(-23i64, List::cons(48i64, List::cons(22i64, List::cons(-9i64, List::cons(1i64, List::cons(5i64, List::<i64>::nil())))))))))), List::cons(List::cons(-1i64, List::cons(40i64, List::cons(2i64, List::cons(-13i64, List::cons(1i64, List::cons(30i64, List::cons(9i64, List::cons(-6i64, List::cons(-16i64, List::cons(-7i64, List::<i64>::nil())))))))))), List::cons(List::cons(4i64, List::cons(-9i64, List::cons(-12i64, List::cons(-14i64, List::cons(-16i64, List::cons(-14i64, List::cons(22i64, List::cons(10i64, List::cons(9i64, List::cons(22i64, List::<i64>::nil())))))))))), List::cons(List::cons(-1i64, List::cons(-22i64, List::cons(0i64, List::cons(-15i64, List::cons(16i64, List::cons(29i64, List::cons(18i64, List::cons(24i64, List::cons(-21i64, List::cons(34i64, List::<i64>::nil())))))))))), List::cons(List::cons(-42i64, List::cons(4i64, List::cons(18i64, List::cons(107i64, List::cons(-44i64, List::cons(-23i64, List::cons(-22i64, List::cons(9i64, List::cons(-29i64, List::cons(-18i64, List::<i64>::nil())))))))))), List::cons(List::cons(34i64, List::cons(46i64, List::cons(31i64, List::cons(-25i64, List::cons(-50i64, List::cons(-22i64, List::cons(-60i64, List::cons(89i64, List::cons(-50i64, List::cons(-39i64, List::<i64>::nil())))))))))), List::cons(List::cons(26i64, List::cons(56i64, List::cons(30i64, List::cons(-76i64, List::cons(-14i64, List::cons(-15i64, List::cons(35i64, List::cons(29i64, List::cons(-19i64, List::cons(-29i64, List::<i64>::nil())))))))))), List::cons(List::cons(46i64, List::cons(-18i64, List::cons(12i64, List::cons(-98i64, List::cons(11i64, List::cons(-12i64, List::cons(46i64, List::cons(-6i64, List::cons(5i64, List::cons(5i64, List::<i64>::nil())))))))))), List::cons(List::cons(-9i64, List::cons(60i64, List::cons(20i64, List::cons(-11i64, List::cons(9i64, List::cons(0i64, List::cons(-17i64, List::cons(15i64, List::cons(26i64, List::cons(-28i64, List::<i64>::nil())))))))))), List::cons(List::cons(-28i64, List::cons(39i64, List::cons(-1i64, List::cons(-6i64, List::cons(-21i64, List::cons(7i64, List::cons(-1i64, List::cons(71i64, List::cons(-22i64, List::cons(-22i64, List::<i64>::nil())))))))))), List::cons(List::cons(23i64, List::cons(0i64, List::cons(-46i64, List::cons(-63i64, List::cons(-20i64, List::cons(-27i64, List::cons(21i64, List::cons(-3i64, List::cons(3i64, List::cons(-5i64, List::<i64>::nil())))))))))), List::cons(List::cons(15i64, List::cons(-1i64, List::cons(46i64, List::cons(-20i64, List::cons(7i64, List::cons(-12i64, List::cons(12i64, List::cons(-69i64, List::cons(19i64, List::cons(-8i64, List::<i64>::nil())))))))))), List::cons(List::cons(-75i64, List::cons(29i64, List::cons(-1i64, List::cons(-124i64, List::cons(58i64, List::cons(-29i64, List::cons(12i64, List::cons(58i64, List::cons(-2i64, List::cons(3i64, List::<i64>::nil())))))))))), List::cons(List::cons(-19i64, List::cons(-31i64, List::cons(14i64, List::cons(69i64, List::cons(52i64, List::cons(0i64, List::cons(6i64, List::cons(-13i64, List::cons(-25i64, List::cons(14i64, List::<i64>::nil())))))))))), List::cons(List::cons(56i64, List::cons(-26i64, List::cons(-23i64, List::cons(26i64, List::cons(-19i64, List::cons(45i64, List::cons(2i64, List::cons(-28i64, List::cons(-11i64, List::cons(-17i64, List::<i64>::nil())))))))))), List::<List<i64>>::nil()))))))))))))))))))))))))))))))));
            append_list(a44, a45)
        };
        append_list(a46, a47)
    };
    let a55 = {
        let a52 = {
            let a48 = List::cons(List::cons(-6i64, List::cons(-44i64, List::cons(30i64, List::cons(0i64, List::cons(-2i64, List::cons(-9i64, List::cons(31i64, List::cons(-35i64, List::cons(-1i64, List::cons(18i64, List::<i64>::nil())))))))))), List::cons(List::cons(-55i64, List::cons(-12i64, List::cons(48i64, List::cons(44i64, List::cons(-49i64, List::cons(-13i64, List::cons(33i64, List::cons(-82i64, List::cons(31i64, List::cons(24i64, List::<i64>::nil())))))))))), List::cons(List::cons(47i64, List::cons(-8i64, List::cons(13i64, List::cons(-37i64, List::cons(-20i64, List::cons(-50i64, List::cons(67i64, List::cons(-60i64, List::cons(-11i64, List::cons(68i64, List::<i64>::nil())))))))))), List::cons(List::cons(28i64, List::cons(52i64, List::cons(-14i64, List::cons(-51i64, List::cons(40i64, List::cons(10i64, List::cons(46i64, List::cons(-17i64, List::cons(-16i64, List::cons(-27i64, List::<i64>::nil())))))))))), List::cons(List::cons(8i64, List::cons(26i64, List::cons(-40i64, List::cons(-31i64, List::cons(34i64, List::cons(-34i64, List::cons(27i64, List::cons(21i64, List::cons(-3i64, List::cons(8i64, List::<i64>::nil())))))))))), List::cons(List::cons(3i64, List::cons(13i64, List::cons(-11i64, List::cons(-3i64, List::cons(24i64, List::cons(-19i64, List::cons(-22i64, List::cons(-27i64, List::cons(3i64, List::cons(-28i64, List::<i64>::nil())))))))))), List::cons(List::cons(27i64, List::cons(26i64, List::cons(-27i64, List::cons(79i64, List::cons(-44i64, List::cons(-26i64, List::cons(16i64, List::cons(11i64, List::cons(-46i64, List::cons(-16i64, List::<i64>::nil())))))))))), List::cons(List::cons(12i64, List::cons(23i64, List::cons(58i64, List::cons(2i64, List::cons(-13i64, List::cons(-9i64, List::cons(-35i64, List::cons(4i64, List::cons(20i64, List::cons(-28i64, List::<i64>::nil())))))))))), List::cons(List::cons(4i64, List::cons(-24i64, List::cons(-12i64, List::cons(-125i64, List::cons(58i64, List::cons(22i64, List::cons(36i64, List::cons(-5i64, List::cons(14i64, List::cons(-40i64, List::<i64>::nil())))))))))), List::cons(List::cons(-16i64, List::cons(-28i64, List::cons(18i64, List::cons(-103i64, List::cons(-8i64, List::cons(-46i64, List::cons(-20i64, List::cons(-31i64, List::cons(127i64, List::cons(-35i64, List::<i64>::nil())))))))))), List::cons(List::cons(-23i64, List::cons(-19i64, List::cons(67i64, List::cons(37i64, List::cons(19i64, List::cons(23i64, List::cons(-61i64, List::cons(33i64, List::cons(-20i64, List::cons(11i64, List::<i64>::nil())))))))))), List::cons(List::cons(34i64, List::cons(-18i64, List::cons(-32i64, List::cons(-18i64, List::cons(-14i64, List::cons(-1i64, List::cons(-36i64, List::cons(15i64, List::cons(0i64, List::cons(6i64, List::<i64>::nil())))))))))), List::cons(List::cons(6i64, List::cons(-27i64, List::cons(17i64, List::cons(5i64, List::cons(-18i64, List::cons(11i64, List::cons(-10i64, List::cons(-1i64, List::cons(-11i64, List::cons(9i64, List::<i64>::nil())))))))))), List::cons(List::cons(-14i64, List::cons(-16i64, List::cons(-54i64, List::cons(46i64, List::cons(-6i64, List::cons(36i64, List::cons(22i64, List::cons(18i64, List::cons(-25i64, List::cons(18i64, List::<i64>::nil())))))))))), List::cons(List::cons(-53i64, List::cons(37i64, List::cons(-13i64, List::cons(111i64, List::cons(19i64, List::cons(34i64, List::cons(-30i64, List::cons(36i64, List::cons(-32i64, List::cons(64i64, List::<i64>::nil())))))))))), List::cons(List::cons(-2i64, List::cons(16i64, List::cons(-44i64, List::cons(44i64, List::cons(-8i64, List::cons(23i64, List::cons(-11i64, List::cons(12i64, List::cons(-7i64, List::cons(9i64, List::<i64>::nil())))))))))), List::cons(List::cons(-6i64, List::cons(-10i64, List::cons(-18i64, List::cons(-1i64, List::cons(2i64, List::cons(-4i64, List::cons(43i64, List::cons(-19i64, List::cons(-1i64, List::cons(-40i64, List::<i64>::nil())))))))))), List::cons(List::cons(-1i64, List::cons(46i64, List::cons(-17i64, List::cons(-19i64, List::cons(-9i64, List::cons(32i64, List::cons(46i64, List::cons(-27i64, List::cons(-1i64, List::cons(-22i64, List::<i64>::nil())))))))))), List::cons(List::cons(3i64, List::cons(24i64, List::cons(20i64, List::cons(-33i64, List::cons(20i64, List::cons(7i64, List::cons(37i64, List::cons(-30i64, List::cons(9i64, List::cons(-42i64, List::<i64>::nil())))))))))), List::cons(List::cons(-10i64, List::cons(11i64, List::cons(-5i64, List::cons(8i64, List::cons(-9i64, List::cons(7i64, List::cons(33i64, List::cons(-15i64, List::cons(10i64, List::cons(-14i64, List::<i64>::nil())))))))))), List::cons(List::cons(21i64, List::cons(-24i64, List::cons(-48i64, List::cons(-15i64, List::cons(-18i64, List::cons(60i64, List::cons(19i64, List::cons(-7i64, List::cons(20i64, List::cons(-15i64, List::<i64>::nil())))))))))), List::cons(List::cons(-9i64, List::cons(20i64, List::cons(-12i64, List::cons(-23i64, List::cons(35i64, List::cons(40i64, List::cons(15i64, List::cons(-41i64, List::cons(2i64, List::cons(24i64, List::<i64>::nil())))))))))), List::cons(List::cons(43i64, List::cons(-20i64, List::cons(25i64, List::cons(-10i64, List::cons(22i64, List::cons(-24i64, List::cons(-34i64, List::cons(56i64, List::cons(-40i64, List::cons(-60i64, List::<i64>::nil())))))))))), List::cons(List::cons(28i64, List::cons(-43i64, List::cons(-35i64, List::cons(45i64, List::cons(-21i64, List::cons(-9i64, List::cons(21i64, List::cons(-2i64, List::cons(7i64, List::cons(-39i64, List::<i64>::nil())))))))))), List::cons(List::cons(1i64, List::cons(-76i64, List::cons(-51i64, List::cons(-27i64, List::cons(-3i64, List::cons(-8i64, List::cons(70i64, List::cons(-30i64, List::cons(28i64, List::cons(30i64, List::<i64>::nil())))))))))), List::cons(List::cons(6i64, List::cons(-9i64, List::cons(-38i64, List::cons(13i64, List::cons(-26i64, List::cons(2i64, List::cons(15i64, List::cons(-30i64, List::cons(6i64, List::cons(-27i64, List::<i64>::nil())))))))))), List::cons(List::cons(15i64, List::cons(89i64, List::cons(-49i64, List::cons(-54i64, List::cons(24i64, List::cons(24i64, List::cons(-73i64, List::cons(9i64, List::cons(-16i64, List::cons(38i64, List::<i64>::nil())))))))))), List::cons(List::cons(14i64, List::cons(-36i64, List::cons(-15i64, List::cons(10i64, List::cons(20i64, List::cons(-4i64, List::cons(-16i64, List::cons(-25i64, List::cons(-11i64, List::cons(31i64, List::<i64>::nil())))))))))), List::cons(List::cons(13i64, List::cons(-55i64, List::cons(45i64, List::cons(-7i64, List::cons(22i64, List::cons(-11i64, List::cons(13i64, List::cons(-30i64, List::cons(19i64, List::cons(-51i64, List::<i64>::nil())))))))))), List::cons(List::cons(18i64, List::cons(2i64, List::cons(14i64, List::cons(-61i64, List::cons(-23i64, List::cons(-95i64, List::cons(39i64, List::cons(21i64, List::cons(29i64, List::cons(-7i64, List::<i64>::nil())))))))))), List::cons(List::cons(-127i64, List::cons(11i64, List::cons(28i64, List::cons(-56i64, List::cons(127i64, List::cons(-74i64, List::cons(-66i64, List::cons(99i64, List::cons(-27i64, List::cons(119i64, List::<i64>::nil())))))))))), List::cons(List::cons(-43i64, List::cons(-18i64, List::cons(21i64, List::cons(-13i64, List::cons(38i64, List::cons(5i64, List::cons(-16i64, List::cons(-20i64, List::cons(4i64, List::cons(32i64, List::<i64>::nil())))))))))), List::<List<i64>>::nil()))))))))))))))))))))))))))))))));
            let a49 = List::cons(List::cons(-12i64, List::cons(-11i64, List::cons(-11i64, List::cons(-11i64, List::cons(-29i64, List::cons(-35i64, List::cons(-15i64, List::cons(27i64, List::cons(4i64, List::cons(14i64, List::<i64>::nil())))))))))), List::cons(List::cons(8i64, List::cons(-6i64, List::cons(-12i64, List::cons(24i64, List::cons(1i64, List::cons(-32i64, List::cons(28i64, List::cons(-14i64, List::cons(6i64, List::cons(38i64, List::<i64>::nil())))))))))), List::cons(List::cons(-52i64, List::cons(24i64, List::cons(9i64, List::cons(89i64, List::cons(2i64, List::cons(-34i64, List::cons(17i64, List::cons(5i64, List::cons(-24i64, List::cons(-18i64, List::<i64>::nil())))))))))), List::cons(List::cons(-49i64, List::cons(1i64, List::cons(-3i64, List::cons(6i64, List::cons(-5i64, List::cons(33i64, List::cons(0i64, List::cons(-29i64, List::cons(5i64, List::cons(-21i64, List::<i64>::nil())))))))))), List::cons(List::cons(-16i64, List::cons(-23i64, List::cons(11i64, List::cons(-11i64, List::cons(-22i64, List::cons(-24i64, List::cons(4i64, List::cons(4i64, List::cons(8i64, List::cons(-28i64, List::<i64>::nil())))))))))), List::cons(List::cons(38i64, List::cons(41i64, List::cons(-20i64, List::cons(13i64, List::cons(-21i64, List::cons(-43i64, List::cons(-13i64, List::cons(0i64, List::cons(-2i64, List::cons(-33i64, List::<i64>::nil())))))))))), List::cons(List::cons(-32i64, List::cons(30i64, List::cons(35i64, List::cons(-5i64, List::cons(-3i64, List::cons(25i64, List::cons(17i64, List::cons(77i64, List::cons(-18i64, List::cons(-38i64, List::<i64>::nil())))))))))), List::cons(List::cons(3i64, List::cons(-47i64, List::cons(-16i64, List::cons(44i64, List::cons(43i64, List::cons(40i64, List::cons(25i64, List::cons(1i64, List::cons(13i64, List::cons(-21i64, List::<i64>::nil())))))))))), List::cons(List::cons(3i64, List::cons(-28i64, List::cons(14i64, List::cons(-49i64, List::cons(14i64, List::cons(-8i64, List::cons(-7i64, List::cons(-1i64, List::cons(-12i64, List::cons(-19i64, List::<i64>::nil())))))))))), List::cons(List::cons(-27i64, List::cons(49i64, List::cons(-19i64, List::cons(-38i64, List::cons(-3i64, List::cons(-3i64, List::cons(2i64, List::cons(4i64, List::cons(16i64, List::cons(28i64, List::<i64>::nil())))))))))), List::cons(List::cons(-36i64, List::cons(3i64, List::cons(-32i64, List::cons(-33i64, List::cons(28i64, List::cons(14i64, List::cons(-21i64, List::cons(-17i64, List::cons(4i64, List::cons(52i64, List::<i64>::nil())))))))))), List::cons(List::cons(-16i64, List::cons(-22i64, List::cons(74i64, List::cons(-42i64, List::cons(39i64, List::cons(-24i64, List::cons(5i64, List::cons(44i64, List::cons(-21i64, List::cons(-22i64, List::<i64>::nil())))))))))), List::cons(List::cons(-5i64, List::cons(-23i64, List::cons(-6i64, List::cons(7i64, List::cons(-10i64, List::cons(-15i64, List::cons(23i64, List::cons(14i64, List::cons(9i64, List::cons(5i64, List::<i64>::nil())))))))))), List::cons(List::cons(2i64, List::cons(7i64, List::cons(-76i64, List::cons(40i64, List::cons(4i64, List::cons(-15i64, List::cons(3i64, List::cons(-8i64, List::cons(8i64, List::cons(23i64, List::<i64>::nil())))))))))), List::cons(List::cons(40i64, List::cons(-4i64, List::cons(11i64, List::cons(-15i64, List::cons(8i64, List::cons(7i64, List::cons(-21i64, List::cons(-15i64, List::cons(-7i64, List::cons(28i64, List::<i64>::nil())))))))))), List::cons(List::cons(-19i64, List::cons(13i64, List::cons(76i64, List::cons(-31i64, List::cons(25i64, List::cons(27i64, List::cons(-13i64, List::cons(-51i64, List::cons(-6i64, List::cons(39i64, List::<i64>::nil())))))))))), List::cons(List::cons(-38i64, List::cons(-24i64, List::cons(-8i64, List::cons(30i64, List::cons(-41i64, List::cons(29i64, List::cons(-14i64, List::cons(30i64, List::cons(25i64, List::cons(-30i64, List::<i64>::nil())))))))))), List::cons(List::cons(10i64, List::cons(40i64, List::cons(-10i64, List::cons(8i64, List::cons(-14i64, List::cons(-2i64, List::cons(-18i64, List::cons(-17i64, List::cons(8i64, List::cons(27i64, List::<i64>::nil())))))))))), List::cons(List::cons(28i64, List::cons(16i64, List::cons(-44i64, List::cons(15i64, List::cons(32i64, List::cons(32i64, List::cons(-57i64, List::cons(-36i64, List::cons(2i64, List::cons(-7i64, List::<i64>::nil())))))))))), List::cons(List::cons(14i64, List::cons(-11i64, List::cons(-7i64, List::cons(-12i64, List::cons(39i64, List::cons(51i64, List::cons(6i64, List::cons(40i64, List::cons(2i64, List::cons(-17i64, List::<i64>::nil())))))))))), List::cons(List::cons(14i64, List::cons(-11i64, List::cons(4i64, List::cons(-87i64, List::cons(57i64, List::cons(-14i64, List::cons(17i64, List::cons(26i64, List::cons(-25i64, List::cons(31i64, List::<i64>::nil())))))))))), List::cons(List::cons(9i64, List::cons(-6i64, List::cons(-16i64, List::cons(-24i64, List::cons(-14i64, List::cons(31i64, List::cons(33i64, List::cons(-28i64, List::cons(-30i64, List::cons(-1i64, List::<i64>::nil())))))))))), List::cons(List::cons(-19i64, List::cons(58i64, List::cons(-5i64, List::cons(-20i64, List::cons(-23i64, List::cons(13i64, List::cons(45i64, List::cons(29i64, List::cons(-27i64, List::cons(-9i64, List::<i64>::nil())))))))))), List::cons(List::cons(12i64, List::cons(-13i64, List::cons(-15i64, List::cons(-49i64, List::cons(-25i64, List::cons(0i64, List::cons(46i64, List::cons(21i64, List::cons(35i64, List::cons(-22i64, List::<i64>::nil())))))))))), List::cons(List::cons(7i64, List::cons(-34i64, List::cons(77i64, List::cons(-54i64, List::cons(-13i64, List::cons(-28i64, List::cons(13i64, List::cons(-55i64, List::cons(-7i64, List::cons(1i64, List::<i64>::nil())))))))))), List::cons(List::cons(-3i64, List::cons(52i64, List::cons(76i64, List::cons(-61i64, List::cons(-25i64, List::cons(-14i64, List::cons(41i64, List::cons(-9i64, List::cons(54i64, List::cons(-87i64, List::<i64>::nil())))))))))), List::cons(List::cons(2i64, List::cons(-2i64, List::cons(63i64, List::cons(-34i64, List::cons(4i64, List::cons(-47i64, List::cons(23i64, List::cons(2i64, List::cons(-24i64, List::cons(64i64, List::<i64>::nil())))))))))), List::cons(List::cons(14i64, List::cons(-11i64, List::cons(-46i64, List::cons(33i64, List::cons(-21i64, List::cons(19i64, List::cons(-15i64, List::cons(-14i64, List::cons(1i64, List::cons(-24i64, List::<i64>::nil())))))))))), List::cons(List::cons(-20i64, List::cons(19i64, List::cons(-20i64, List::cons(85i64, List::cons(1i64, List::cons(-9i64, List::cons(-38i64, List::cons(26i64, List::cons(14i64, List::cons(19i64, List::<i64>::nil())))))))))), List::cons(List::cons(-5i64, List::cons(-23i64, List::cons(35i64, List::cons(-6i64, List::cons(-37i64, List::cons(18i64, List::cons(-38i64, List::cons(21i64, List::cons(-15i64, List::cons(7i64, List::<i64>::nil())))))))))), List::cons(List::cons(-17i64, List::cons(14i64, List::cons(5i64, List::cons(19i64, List::cons(-32i64, List::cons(18i64, List::cons(-5i64, List::cons(-25i64, List::cons(-3i64, List::cons(40i64, List::<i64>::nil())))))))))), List::cons(List::cons(-2i64, List::cons(-22i64, List::cons(-15i64, List::cons(3i64, List::cons(46i64, List::cons(-20i64, List::cons(-17i64, List::cons(7i64, List::cons(-5i64, List::cons(-36i64, List::<i64>::nil())))))))))), List::<List<i64>>::nil()))))))))))))))))))))))))))))))));
            append_list(a48, a49)
        };
        let a53 = {
            let a50 = List::cons(List::cons(20i64, List::cons(12i64, List::cons(-24i64, List::cons(-6i64, List::cons(15i64, List::cons(-19i64, List::cons(-4i64, List::cons(9i64, List::cons(-1i64, List::cons(18i64, List::<i64>::nil())))))))))), List::cons(List::cons(14i64, List::cons(-33i64, List::cons(12i64, List::cons(18i64, List::cons(-15i64, List::cons(17i64, List::cons(-48i64, List::cons(-2i64, List::cons(17i64, List::cons(8i64, List::<i64>::nil())))))))))), List::cons(List::cons(0i64, List::cons(-15i64, List::cons(-31i64, List::cons(1i64, List::cons(-34i64, List::cons(11i64, List::cons(28i64, List::cons(-27i64, List::cons(-11i64, List::cons(11i64, List::<i64>::nil())))))))))), List::cons(List::cons(-14i64, List::cons(14i64, List::cons(-43i64, List::cons(-40i64, List::cons(-25i64, List::cons(41i64, List::cons(51i64, List::cons(24i64, List::cons(3i64, List::cons(-32i64, List::<i64>::nil())))))))))), List::cons(List::cons(-18i64, List::cons(25i64, List::cons(32i64, List::cons(26i64, List::cons(1i64, List::cons(-8i64, List::cons(-22i64, List::cons(1i64, List::cons(-13i64, List::cons(-9i64, List::<i64>::nil())))))))))), List::cons(List::cons(-10i64, List::cons(7i64, List::cons(45i64, List::cons(-1i64, List::cons(-16i64, List::cons(-6i64, List::cons(18i64, List::cons(12i64, List::cons(-2i64, List::cons(-35i64, List::<i64>::nil())))))))))), List::cons(List::cons(27i64, List::cons(111i64, List::cons(21i64, List::cons(-25i64, List::cons(-37i64, List::cons(-3i64, List::cons(16i64, List::cons(-26i64, List::cons(-22i64, List::cons(-16i64, List::<i64>::nil())))))))))), List::cons(List::cons(-54i64, List::cons(35i64, List::cons(-23i64, List::cons(24i64, List::cons(0i64, List::cons(33i64, List::cons(50i64, List::cons(27i64, List::cons(-20i64, List::cons(-14i64, List::<i64>::nil())))))))))), List::cons(List::cons(-43i64, List::cons(22i64, List::cons(-15i64, List::cons(4i64, List::cons(-11i64, List::cons(26i64, List::cons(-44i64, List::cons(41i64, List::cons(22i64, List::cons(-7i64, List::<i64>::nil())))))))))), List::cons(List::cons(44i64, List::cons(39i64, List::cons(22i64, List::cons(-69i64, List::cons(-4i64, List::cons(-46i64, List::cons(-40i64, List::cons(50i64, List::cons(-17i64, List::cons(-1i64, List::<i64>::nil())))))))))), List::cons(List::cons(12i64, List::cons(71i64, List::cons(12i64, List::cons(0i64, List::cons(-18i64, List::cons(31i64, List::cons(0i64, List::cons(46i64, List::cons(-11i64, List::cons(-45i64, List::<i64>::nil())))))))))), List::cons(List::cons(-16i64, List::cons(-78i64, List::cons(-50i64, List::cons(77i64, List::cons(-11i64, List::cons(-59i64, List::cons(-39i64, List::cons(-9i64, List::cons(51i64, List::cons(53i64, List::<i64>::nil())))))))))), List::cons(List::cons(1i64, List::cons(-85i64, List::cons(-17i64, List::cons(32i64, List::cons(5i64, List::cons(16i64, List::cons(16i64, List::cons(-4i64, List::cons(-10i64, List::cons(44i64, List::<i64>::nil())))))))))), List::cons(List::cons(33i64, List::cons(-2i64, List::cons(81i64, List::cons(-40i64, List::cons(18i64, List::cons(-17i64, List::cons(29i64, List::cons(-40i64, List::cons(-23i64, List::cons(-33i64, List::<i64>::nil())))))))))), List::cons(List::cons(21i64, List::cons(15i64, List::cons(24i64, List::cons(-48i64, List::cons(-8i64, List::cons(-4i64, List::cons(0i64, List::cons(24i64, List::cons(-27i64, List::cons(3i64, List::<i64>::nil())))))))))), List::cons(List::cons(59i64, List::cons(-76i64, List::cons(3i64, List::cons(67i64, List::cons(10i64, List::cons(9i64, List::cons(38i64, List::cons(-21i64, List::cons(-8i64, List::cons(-13i64, List::<i64>::nil())))))))))), List::cons(List::cons(-18i64, List::cons(-16i64, List::cons(-6i64, List::cons(14i64, List::cons(19i64, List::cons(-26i64, List::cons(29i64, List::cons(6i64, List::cons(12i64, List::cons(-7i64, List::<i64>::nil())))))))))), List::cons(List::cons(0i64, List::cons(67i64, List::cons(-28i64, List::cons(-62i64, List::cons(59i64, List::cons(29i64, List::cons(32i64, List::cons(25i64, List::cons(-19i64, List::cons(-41i64, List::<i64>::nil())))))))))), List::cons(List::cons(-11i64, List::cons(32i64, List::cons(-5i64, List::cons(-82i64, List::cons(60i64, List::cons(16i64, List::cons(-13i64, List::cons(-12i64, List::cons(1i64, List::cons(-72i64, List::<i64>::nil())))))))))), List::cons(List::cons(-16i64, List::cons(86i64, List::cons(-51i64, List::cons(-25i64, List::cons(39i64, List::cons(15i64, List::cons(16i64, List::cons(-14i64, List::cons(-7i64, List::cons(-54i64, List::<i64>::nil())))))))))), List::cons(List::cons(36i64, List::cons(-29i64, List::cons(-39i64, List::cons(-39i64, List::cons(10i64, List::cons(-8i64, List::cons(38i64, List::cons(-35i64, List::cons(22i64, List::cons(14i64, List::<i64>::nil())))))))))), List::cons(List::cons(38i64, List::cons(-127i64, List::cons(-8i64, List::cons(-51i64, List::cons(55i64, List::cons(27i64, List::cons(51i64, List::cons(-46i64, List::cons(5i64, List::cons(-3i64, List::<i64>::nil())))))))))), List::cons(List::cons(-23i64, List::cons(3i64, List::cons(2i64, List::cons(117i64, List::cons(-70i64, List::cons(-48i64, List::cons(10i64, List::cons(45i64, List::cons(9i64, List::cons(-41i64, List::<i64>::nil())))))))))), List::cons(List::cons(20i64, List::cons(20i64, List::cons(81i64, List::cons(39i64, List::cons(-10i64, List::cons(-127i64, List::cons(-46i64, List::cons(-24i64, List::cons(30i64, List::cons(23i64, List::<i64>::nil())))))))))), List::cons(List::cons(47i64, List::cons(-31i64, List::cons(-60i64, List::cons(2i64, List::cons(-8i64, List::cons(-21i64, List::cons(40i64, List::cons(-12i64, List::cons(27i64, List::cons(20i64, List::<i64>::nil())))))))))), List::cons(List::cons(49i64, List::cons(-69i64, List::cons(19i64, List::cons(-112i64, List::cons(59i64, List::cons(-36i64, List::cons(-31i64, List::cons(-8i64, List::cons(17i64, List::cons(61i64, List::<i64>::nil())))))))))), List::cons(List::cons(-17i64, List::cons(-10i64, List::cons(32i64, List::cons(-29i64, List::cons(-9i64, List::cons(-2i64, List::cons(-35i64, List::cons(-35i64, List::cons(-19i64, List::cons(6i64, List::<i64>::nil())))))))))), List::cons(List::cons(-10i64, List::cons(-1i64, List::cons(-31i64, List::cons(26i64, List::cons(-15i64, List::cons(18i64, List::cons(24i64, List::cons(38i64, List::cons(3i64, List::cons(-9i64, List::<i64>::nil())))))))))), List::cons(List::cons(27i64, List::cons(-32i64, List::cons(27i64, List::cons(-33i64, List::cons(-27i64, List::cons(8i64, List::cons(-16i64, List::cons(16i64, List::cons(-3i64, List::cons(-7i64, List::<i64>::nil())))))))))), List::cons(List::cons(-30i64, List::cons(62i64, List::cons(-80i64, List::cons(-64i64, List::cons(43i64, List::cons(-16i64, List::cons(-37i64, List::cons(75i64, List::cons(-11i64, List::cons(66i64, List::<i64>::nil())))))))))), List::cons(List::cons(-94i64, List::cons(13i64, List::cons(-25i64, List::cons(29i64, List::cons(78i64, List::cons(9i64, List::cons(-33i64, List::cons(93i64, List::cons(-37i64, List::cons(84i64, List::<i64>::nil())))))))))), List::cons(List::cons(-45i64, List::cons(-17i64, List::cons(-63i64, List::cons(67i64, List::cons(27i64, List::cons(18i64, List::cons(-2i64, List::cons(10i64, List::cons(24i64, List::cons(6i64, List::<i64>::nil())))))))))), List::<List<i64>>::nil()))))))))))))))))))))))))))))))));
            let a51 = List::cons(List::cons(-9i64, List::cons(-1i64, List::cons(-22i64, List::cons(-22i64, List::cons(5i64, List::cons(-6i64, List::cons(-15i64, List::cons(-6i64, List::cons(-11i64, List::cons(-16i64, List::<i64>::nil())))))))))), List::cons(List::cons(-17i64, List::cons(71i64, List::cons(-44i64, List::cons(11i64, List::cons(17i64, List::cons(1i64, List::cons(-28i64, List::cons(17i64, List::cons(-6i64, List::cons(-35i64, List::<i64>::nil())))))))))), List::cons(List::cons(20i64, List::cons(57i64, List::cons(-22i64, List::cons(2i64, List::cons(60i64, List::cons(-54i64, List::cons(26i64, List::cons(5i64, List::cons(10i64, List::cons(-77i64, List::<i64>::nil())))))))))), List::cons(List::cons(-37i64, List::cons(-1i64, List::cons(24i64, List::cons(47i64, List::cons(34i64, List::cons(-26i64, List::cons(27i64, List::cons(-1i64, List::cons(-11i64, List::cons(13i64, List::<i64>::nil())))))))))), List::cons(List::cons(-24i64, List::cons(14i64, List::cons(11i64, List::cons(-4i64, List::cons(16i64, List::cons(-10i64, List::cons(3i64, List::cons(23i64, List::cons(-13i64, List::cons(-5i64, List::<i64>::nil())))))))))), List::cons(List::cons(12i64, List::cons(11i64, List::cons(-15i64, List::cons(-42i64, List::cons(7i64, List::cons(23i64, List::cons(-4i64, List::cons(7i64, List::cons(-29i64, List::cons(-28i64, List::<i64>::nil())))))))))), List::cons(List::cons(23i64, List::cons(43i64, List::cons(-4i64, List::cons(-3i64, List::cons(25i64, List::cons(-21i64, List::cons(-46i64, List::cons(29i64, List::cons(-33i64, List::cons(-22i64, List::<i64>::nil())))))))))), List::cons(List::cons(8i64, List::cons(-35i64, List::cons(-3i64, List::cons(11i64, List::cons(23i64, List::cons(-20i64, List::cons(-26i64, List::cons(-29i64, List::cons(-8i64, List::cons(42i64, List::<i64>::nil())))))))))), List::cons(List::cons(-1i64, List::cons(-39i64, List::cons(2i64, List::cons(-12i64, List::cons(-18i64, List::cons(-5i64, List::cons(12i64, List::cons(22i64, List::cons(-12i64, List::cons(-6i64, List::<i64>::nil())))))))))), List::cons(List::cons(-24i64, List::cons(-3i64, List::cons(-19i64, List::cons(81i64, List::cons(-34i64, List::cons(26i64, List::cons(-19i64, List::cons(35i64, List::cons(-15i64, List::cons(-3i64, List::<i64>::nil())))))))))), List::cons(List::cons(9i64, List::cons(25i64, List::cons(-10i64, List::cons(-62i64, List::cons(-32i64, List::cons(48i64, List::cons(-8i64, List::cons(22i64, List::cons(-4i64, List::cons(17i64, List::<i64>::nil())))))))))), List::cons(List::cons(-18i64, List::cons(-79i64, List::cons(-67i64, List::cons(116i64, List::cons(-73i64, List::cons(88i64, List::cons(40i64, List::cons(-57i64, List::cons(43i64, List::cons(-26i64, List::<i64>::nil())))))))))), List::cons(List::cons(-17i64, List::cons(-12i64, List::cons(-14i64, List::cons(-16i64, List::cons(26i64, List::cons(34i64, List::cons(-29i64, List::cons(-16i64, List::cons(-13i64, List::cons(3i64, List::<i64>::nil())))))))))), List::cons(List::cons(28i64, List::cons(-15i64, List::cons(10i64, List::cons(-9i64, List::cons(23i64, List::cons(-30i64, List::cons(-32i64, List::cons(10i64, List::cons(29i64, List::cons(-5i64, List::<i64>::nil())))))))))), List::cons(List::cons(-22i64, List::cons(38i64, List::cons(-25i64, List::cons(28i64, List::cons(-28i64, List::cons(15i64, List::cons(-49i64, List::cons(21i64, List::cons(-10i64, List::cons(23i64, List::<i64>::nil())))))))))), List::cons(List::cons(-34i64, List::cons(1i64, List::cons(31i64, List::cons(6i64, List::cons(-2i64, List::cons(4i64, List::cons(-17i64, List::cons(-31i64, List::cons(50i64, List::cons(-27i64, List::<i64>::nil())))))))))), List::cons(List::cons(17i64, List::cons(-4i64, List::cons(-12i64, List::cons(-9i64, List::cons(1i64, List::cons(-10i64, List::cons(-20i64, List::cons(-21i64, List::cons(6i64, List::cons(-24i64, List::<i64>::nil())))))))))), List::cons(List::cons(-18i64, List::cons(9i64, List::cons(19i64, List::cons(-26i64, List::cons(29i64, List::cons(-33i64, List::cons(45i64, List::cons(29i64, List::cons(-2i64, List::cons(-32i64, List::<i64>::nil())))))))))), List::cons(List::cons(-25i64, List::cons(32i64, List::cons(-39i64, List::cons(24i64, List::cons(9i64, List::cons(-47i64, List::cons(68i64, List::cons(-1i64, List::cons(-13i64, List::cons(-23i64, List::<i64>::nil())))))))))), List::cons(List::cons(-17i64, List::cons(11i64, List::cons(-28i64, List::cons(12i64, List::cons(11i64, List::cons(4i64, List::cons(0i64, List::cons(-5i64, List::cons(-6i64, List::cons(11i64, List::<i64>::nil())))))))))), List::cons(List::cons(6i64, List::cons(-16i64, List::cons(-2i64, List::cons(-22i64, List::cons(-10i64, List::cons(-13i64, List::cons(-11i64, List::cons(-19i64, List::cons(15i64, List::cons(18i64, List::<i64>::nil())))))))))), List::cons(List::cons(-29i64, List::cons(-42i64, List::cons(-51i64, List::cons(-14i64, List::cons(44i64, List::cons(-35i64, List::cons(69i64, List::cons(4i64, List::cons(6i64, List::cons(36i64, List::<i64>::nil())))))))))), List::cons(List::cons(44i64, List::cons(15i64, List::cons(1i64, List::cons(-111i64, List::cons(-6i64, List::cons(24i64, List::cons(48i64, List::cons(5i64, List::cons(-40i64, List::cons(2i64, List::<i64>::nil())))))))))), List::cons(List::cons(-49i64, List::cons(58i64, List::cons(-32i64, List::cons(14i64, List::cons(72i64, List::cons(-70i64, List::cons(-20i64, List::cons(46i64, List::cons(8i64, List::cons(-18i64, List::<i64>::nil())))))))))), List::cons(List::cons(10i64, List::cons(-6i64, List::cons(-5i64, List::cons(-21i64, List::cons(-7i64, List::cons(12i64, List::cons(3i64, List::cons(22i64, List::cons(-14i64, List::cons(17i64, List::<i64>::nil())))))))))), List::cons(List::cons(67i64, List::cons(-48i64, List::cons(-55i64, List::cons(-56i64, List::cons(-4i64, List::cons(-35i64, List::cons(7i64, List::cons(88i64, List::cons(-3i64, List::cons(6i64, List::<i64>::nil())))))))))), List::cons(List::cons(-11i64, List::cons(81i64, List::cons(14i64, List::cons(-47i64, List::cons(-54i64, List::cons(38i64, List::cons(7i64, List::cons(-6i64, List::cons(27i64, List::cons(-44i64, List::<i64>::nil())))))))))), List::cons(List::cons(51i64, List::cons(-29i64, List::cons(57i64, List::cons(-87i64, List::cons(-4i64, List::cons(28i64, List::cons(-61i64, List::cons(68i64, List::cons(-1i64, List::cons(-43i64, List::<i64>::nil())))))))))), List::cons(List::cons(-5i64, List::cons(-27i64, List::cons(-23i64, List::cons(13i64, List::cons(-14i64, List::cons(17i64, List::cons(2i64, List::cons(1i64, List::cons(1i64, List::cons(-1i64, List::<i64>::nil())))))))))), List::cons(List::cons(-30i64, List::cons(99i64, List::cons(-26i64, List::cons(-46i64, List::cons(11i64, List::cons(30i64, List::cons(-11i64, List::cons(0i64, List::cons(-20i64, List::cons(-19i64, List::<i64>::nil())))))))))), List::cons(List::cons(-17i64, List::cons(22i64, List::cons(-33i64, List::cons(1i64, List::cons(60i64, List::cons(21i64, List::cons(-80i64, List::cons(10i64, List::cons(-40i64, List::cons(87i64, List::<i64>::nil())))))))))), List::cons(List::cons(22i64, List::cons(-26i64, List::cons(-23i64, List::cons(45i64, List::cons(-45i64, List::cons(50i64, List::cons(-38i64, List::cons(40i64, List::cons(-4i64, List::cons(20i64, List::<i64>::nil())))))))))), List::<List<i64>>::nil()))))))))))))))))))))))))))))))));
            append_list(a50, a51)
        };
        append_list(a52, a53)
    };
    append_list(a54, a55)
}

pub fn f17() -> List<i64> {
    List::cons(4170290i64, List::cons(3228102i64, List::cons(4296960i64, List::cons(3259225i64, List::cons(4310572i64, List::cons(3806946i64, List::cons(4095939i64, List::cons(4162557i64, List::cons(6617038i64, List::cons(4237083i64, List::<i64>::nil()))))))))))
}

pub fn f18() -> List<i64> {
    List::cons(78i64, List::cons(128i64, List::cons(186i64, List::cons(150i64, List::cons(0i64, List::cons(255i64, List::cons(68i64, List::cons(74i64, List::cons(202i64, List::cons(69i64, List::<i64>::nil()))))))))))
}

pub fn f19(v0: List<i64>, v1: List<i64>, v2: i64, v3: i64) -> R<i64> {
    let v4: i64 = f35(v2, f36(), 0i64, f37(v0, v1)?)?;
    let v5: i64 = {
        let a71 = {
            let a63 = {
                let a58 = {
                    let a56 = v4;
                    let a57 = f36();
                    int_sub(a56, a57)?
                };
                let a59 = f38();
                int_mul(a58, a59)?
            };
            let a64 = {
                let a62 = {
                    let a60 = v3;
                    let a61 = f39();
                    int_mul(a60, a61)?
                };
                int_neg(a62)?
            };
            int_sub(a63, a64)?
        };
        let a72 = {
            let a70 = {
                let a68 = 2147483648i64;
                let a69 = {
                    let a67 = {
                        let a65 = f40();
                        let a66 = 4294967296i64;
                        int_mul(a65, a66)?
                    };
                    int_neg(a67)?
                };
                int_sub(a68, a69)?
            };
            int_neg(a70)?
        };
        int_sub(a71, a72)?
    };
    let v6: i64 = {
        let a73 = v5;
        let a74 = 4294967296i64;
        let a75 = 0i64;
        int_quot(a73, a74, a75)?
    };
    let v7: i64 = {
        let m78 = {
            let a76 = v6;
            let a77 = 0i64;
            compare(a76, a77)
        };
        if match m78 {
            Ordering::Less => {
                true
            }
            Ordering::Equal => {
                false
            }
            Ordering::Greater => {
                false
            }
        } {
            0i64
        } else {
            v6
        }
    };
    let v8: i64 = {
        let m81 = {
            let a79 = 255i64;
            let a80 = v7;
            compare(a79, a80)
        };
        if match m81 {
            Ordering::Less => {
                true
            }
            Ordering::Equal => {
                false
            }
            Ordering::Greater => {
                false
            }
        } {
            255i64
        } else {
            v7
        }
    };
    Ok(v8)
}

pub fn f20(v0: List<List<i64>>, v1: u64) -> List<i64> {
    let m84 = {
        let a82 = v0;
        let a83 = v1;
        index_list(a82, a83)
    };
    match m84 {
        None => {
            List::<i64>::nil()
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f21(v0: List<i64>, v1: u64) -> i64 {
    let m87 = {
        let a85 = v0;
        let a86 = v1;
        index_list(a85, a86)
    };
    match m87 {
        None => {
            0i64
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f22(v0: List<i64>, v1: u64) -> R<i64> {
    let a88 = f21(v0, v1);
    let a89 = f41();
    int_sub(a88, a89)
}

pub fn f23(v0: Fn2, v1: List<List<i64>>, v2: List<List<i64>>) -> R<List<List<i64>>> {
    let m90 = v2;
    match m90.uncons() {
        None => {
            Ok(v1)
        }
        Some((v3, v4)) => {
            f23(v0.clone(), {
                let c91: Fn2 = v0;
                c91.apply(v1, v3)?
            }, v4)
        }
    }
}

pub fn f24(v0: List<List<i64>>, v1: List<i64>) -> List<List<i64>> {
    List::cons(v1, v0)
}

pub fn f25(v0: Bytes, v1: u64) -> R<List<i64>> {
    f34(f42(Fn4::F43(v0, v1), List::<i64>::nil(), f11())?)
}

pub fn f26(v0: List<List<List<i64>>>) -> List<i64> {
    f44(Fn5::F45, List::<i64>::nil(), v0)
}

pub fn f27(v0: List<List<List<i64>>>) -> R<List<List<List<i64>>>> {
    f46(f47(Fn6::F48, List::<List<List<i64>>>::nil(), v0)?)
}

pub fn f28(v0: List<List<List<i64>>>) -> R<List<List<List<i64>>>> {
    let v1: List<List<List<List<i64>>>> = f49();
    let v2: List<i64> = f50();
    let v3: List<i64> = f51();
    let m92 = v0;
    match m92.uncons() {
        None => {
            Ok(List::<List<List<i64>>>::nil())
        }
        Some((v4, v5)) => {
            let m93 = v5;
            match m93.uncons() {
                None => {
                    Ok(List::<List<List<i64>>>::nil())
                }
                Some((v6, v7)) => {
                    let m94 = v7;
                    match m94.uncons() {
                        None => {
                            Ok(List::<List<List<i64>>>::nil())
                        }
                        Some((v8, v9)) => {
                            let m95 = v9;
                            match m95.uncons() {
                                None => {
                                    Ok(List::<List<List<i64>>>::nil())
                                }
                                Some((v10, v11)) => {
                                    let m96 = v11;
                                    match m96.uncons() {
                                        None => {
                                            Ok(List::<List<List<i64>>>::nil())
                                        }
                                        Some((v12, v13)) => {
                                            let m97 = v13;
                                            match m97.uncons() {
                                                None => {
                                                    Ok(List::<List<List<i64>>>::nil())
                                                }
                                                Some((v14, v15)) => {
                                                    let m98 = v15;
                                                    match m98.uncons() {
                                                        None => {
                                                            Ok(List::<List<List<i64>>>::nil())
                                                        }
                                                        Some((v16, v17)) => {
                                                            let m99 = v17;
                                                            match m99.uncons() {
                                                                None => {
                                                                    Ok(List::<List<List<i64>>>::nil())
                                                                }
                                                                Some((v18, _)) => {
                                                                    Ok(List::cons(f52(f53(v1.clone(), 0u64), f21(v3.clone(), 0u64), f21(v2.clone(), 0u64), v4.clone(), v6.clone(), v8.clone(), v10.clone(), v12.clone(), v14.clone(), v16.clone(), v18.clone())?, List::cons(f52(f53(v1.clone(), 1u64), f21(v3.clone(), 1u64), f21(v2.clone(), 1u64), v4.clone(), v6.clone(), v8.clone(), v10.clone(), v12.clone(), v14.clone(), v16.clone(), v18.clone())?, List::cons(f52(f53(v1.clone(), 2u64), f21(v3.clone(), 2u64), f21(v2.clone(), 2u64), v4.clone(), v6.clone(), v8.clone(), v10.clone(), v12.clone(), v14.clone(), v16.clone(), v18.clone())?, List::cons(f52(f53(v1.clone(), 3u64), f21(v3.clone(), 3u64), f21(v2.clone(), 3u64), v4.clone(), v6.clone(), v8.clone(), v10.clone(), v12.clone(), v14.clone(), v16.clone(), v18.clone())?, List::cons(f52(f53(v1.clone(), 4u64), f21(v3.clone(), 4u64), f21(v2.clone(), 4u64), v4.clone(), v6.clone(), v8.clone(), v10.clone(), v12.clone(), v14.clone(), v16.clone(), v18.clone())?, List::cons(f52(f53(v1.clone(), 5u64), f21(v3.clone(), 5u64), f21(v2.clone(), 5u64), v4.clone(), v6.clone(), v8.clone(), v10.clone(), v12.clone(), v14.clone(), v16.clone(), v18.clone())?, List::cons(f52(f53(v1.clone(), 6u64), f21(v3.clone(), 6u64), f21(v2.clone(), 6u64), v4.clone(), v6.clone(), v8.clone(), v10.clone(), v12.clone(), v14.clone(), v16.clone(), v18.clone())?, List::cons(f52(f53(v1.clone(), 7u64), f21(v3.clone(), 7u64), f21(v2.clone(), 7u64), v4.clone(), v6.clone(), v8.clone(), v10.clone(), v12.clone(), v14.clone(), v16.clone(), v18.clone())?, List::cons(f52(f53(v1.clone(), 8u64), f21(v3.clone(), 8u64), f21(v2.clone(), 8u64), v4.clone(), v6.clone(), v8.clone(), v10.clone(), v12.clone(), v14.clone(), v16.clone(), v18.clone())?, List::cons(f52(f53(v1.clone(), 9u64), f21(v3.clone(), 9u64), f21(v2.clone(), 9u64), v4.clone(), v6.clone(), v8.clone(), v10.clone(), v12.clone(), v14.clone(), v16.clone(), v18.clone())?, List::cons(f52(f53(v1.clone(), 10u64), f21(v3.clone(), 10u64), f21(v2.clone(), 10u64), v4.clone(), v6.clone(), v8.clone(), v10.clone(), v12.clone(), v14.clone(), v16.clone(), v18.clone())?, List::cons(f52(f53(v1.clone(), 11u64), f21(v3.clone(), 11u64), f21(v2.clone(), 11u64), v4.clone(), v6.clone(), v8.clone(), v10.clone(), v12.clone(), v14.clone(), v16.clone(), v18.clone())?, List::cons(f52(f53(v1.clone(), 12u64), f21(v3.clone(), 12u64), f21(v2.clone(), 12u64), v4.clone(), v6.clone(), v8.clone(), v10.clone(), v12.clone(), v14.clone(), v16.clone(), v18.clone())?, List::cons(f52(f53(v1.clone(), 13u64), f21(v3.clone(), 13u64), f21(v2.clone(), 13u64), v4.clone(), v6.clone(), v8.clone(), v10.clone(), v12.clone(), v14.clone(), v16.clone(), v18.clone())?, List::cons(f52(f53(v1.clone(), 14u64), f21(v3.clone(), 14u64), f21(v2.clone(), 14u64), v4.clone(), v6.clone(), v8.clone(), v10.clone(), v12.clone(), v14.clone(), v16.clone(), v18.clone())?, List::cons(f52(f53(v1, 15u64), f21(v3, 15u64), f21(v2, 15u64), v4, v6, v8, v10, v12, v14, v16, v18)?, List::<List<List<i64>>>::nil())))))))))))))))))
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn f29(v0: List<List<List<i64>>>) -> R<List<List<List<i64>>>> {
    f46(f47(Fn6::F54, List::<List<List<i64>>>::nil(), v0)?)
}

pub fn f30(v0: List<List<List<i64>>>) -> R<List<List<List<i64>>>> {
    f46(f47(Fn6::F55, List::<List<List<i64>>>::nil(), v0)?)
}

pub fn f31(v0: List<List<i64>>) -> R<List<List<List<i64>>>> {
    let v1: List<List<List<List<i64>>>> = f56();
    let v2: List<i64> = f57();
    let v3: List<i64> = f58();
    Ok(List::cons(f59(f60(f61(f53(v1.clone(), 0u64), 0u64)), f21(v3.clone(), 0u64), f21(v2.clone(), 0u64), v0.clone())?, List::cons(f59(f60(f61(f53(v1.clone(), 1u64), 0u64)), f21(v3.clone(), 1u64), f21(v2.clone(), 1u64), v0.clone())?, List::cons(f59(f60(f61(f53(v1.clone(), 2u64), 0u64)), f21(v3.clone(), 2u64), f21(v2.clone(), 2u64), v0.clone())?, List::cons(f59(f60(f61(f53(v1.clone(), 3u64), 0u64)), f21(v3.clone(), 3u64), f21(v2.clone(), 3u64), v0.clone())?, List::cons(f59(f60(f61(f53(v1.clone(), 4u64), 0u64)), f21(v3.clone(), 4u64), f21(v2.clone(), 4u64), v0.clone())?, List::cons(f59(f60(f61(f53(v1.clone(), 5u64), 0u64)), f21(v3.clone(), 5u64), f21(v2.clone(), 5u64), v0.clone())?, List::cons(f59(f60(f61(f53(v1.clone(), 6u64), 0u64)), f21(v3.clone(), 6u64), f21(v2.clone(), 6u64), v0.clone())?, List::cons(f59(f60(f61(f53(v1, 7u64), 0u64)), f21(v3, 7u64), f21(v2, 7u64), v0)?, List::<List<List<i64>>>::nil())))))))))
}

pub fn f32(v0: Fn3, v1: (((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>))), ((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>)))), v2: List<List<i64>>) -> (((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>))), ((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>)))) {
    let m100 = v2;
    match m100.uncons() {
        None => {
            v1
        }
        Some((v3, v4)) => {
            f32(v0.clone(), {
                let c101: Fn3 = v0;
                c101.apply(v1, v3)
            }, v4)
        }
    }
}

pub fn f33(v0: (((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>))), ((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>)))), v1: List<i64>) -> (((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>))), ((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>)))) {
    let v2: ((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>))) = {
        let (h102, _) = v0.clone();
        h102
    };
    let v3: (List<i64>, List<i64>) = {
        let (h103, _) = v2;
        h103
    };
    let v4: (List<i64>, (List<i64>, List<i64>)) = {
        let (_, h104) = v2;
        h104
    };
    let v5: (List<i64>, List<i64>) = {
        let (_, h105) = v4;
        h105
    };
    let v6: ((List<i64>, List<i64>), (List<i64>, (List<i64>, List<i64>))) = {
        let (_, h106) = v0.clone();
        h106
    };
    let v7: (List<i64>, List<i64>) = {
        let (h107, _) = v6;
        h107
    };
    let v8: (List<i64>, (List<i64>, List<i64>)) = {
        let (_, h108) = v6;
        h108
    };
    let v9: (List<i64>, List<i64>) = {
        let (_, h109) = v8;
        h109
    };
    let m110 = v1;
    match m110.uncons() {
        None => {
            v0
        }
        Some((v10, v11)) => {
            let m111 = v11;
            match m111.uncons() {
                None => {
                    v0
                }
                Some((v12, v13)) => {
                    let m112 = v13;
                    match m112.uncons() {
                        None => {
                            v0
                        }
                        Some((v14, v15)) => {
                            let m113 = v15;
                            match m113.uncons() {
                                None => {
                                    v0
                                }
                                Some((v16, v17)) => {
                                    let m114 = v17;
                                    match m114.uncons() {
                                        None => {
                                            v0
                                        }
                                        Some((v18, v19)) => {
                                            let m115 = v19;
                                            match m115.uncons() {
                                                None => {
                                                    v0
                                                }
                                                Some((v20, v21)) => {
                                                    let m116 = v21;
                                                    match m116.uncons() {
                                                        None => {
                                                            v0
                                                        }
                                                        Some((v22, v23)) => {
                                                            let m117 = v23;
                                                            match m117.uncons() {
                                                                None => {
                                                                    v0
                                                                }
                                                                Some((v24, v25)) => {
                                                                    let m118 = v25;
                                                                    match m118.uncons() {
                                                                        None => {
                                                                            v0
                                                                        }
                                                                        Some((v26, v27)) => {
                                                                            let m119 = v27;
                                                                            match m119.uncons() {
                                                                                None => {
                                                                                    v0
                                                                                }
                                                                                Some((v28, _)) => {
                                                                                    (((List::cons(v10, {
                                                                                        let (h120, _) = v3;
                                                                                        h120
                                                                                    }), List::cons(v12, {
                                                                                        let (_, h121) = v3;
                                                                                        h121
                                                                                    })), (List::cons(v14, {
                                                                                        let (h122, _) = v4;
                                                                                        h122
                                                                                    }), (List::cons(v16, {
                                                                                        let (h123, _) = v5;
                                                                                        h123
                                                                                    }), List::cons(v18, {
                                                                                        let (_, h124) = v5;
                                                                                        h124
                                                                                    })))), ((List::cons(v20, {
                                                                                        let (h125, _) = v7;
                                                                                        h125
                                                                                    }), List::cons(v22, {
                                                                                        let (_, h126) = v7;
                                                                                        h126
                                                                                    })), (List::cons(v24, {
                                                                                        let (h127, _) = v8;
                                                                                        h127
                                                                                    }), (List::cons(v26, {
                                                                                        let (h128, _) = v9;
                                                                                        h128
                                                                                    }), List::cons(v28, {
                                                                                        let (_, h129) = v9;
                                                                                        h129
                                                                                    })))))
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn f34(v0: List<i64>) -> R<List<i64>> {
    f62(Fn7::F63, List::<i64>::nil(), v0)
}

pub fn f35(v0: i64, v1: i64, v2: i64, v3: i64) -> R<i64> {
    let v4: i64 = {
        let a141 = {
            let a136 = {
                let a133 = {
                    let a131 = v3;
                    let a132 = {
                        let a130 = v2;
                        int_neg(a130)?
                    };
                    int_sub(a131, a132)?
                };
                let a134 = v0;
                int_mul(a133, a134)?
            };
            let a137 = {
                let a135 = 2147483648i64;
                int_neg(a135)?
            };
            int_sub(a136, a137)?
        };
        let a142 = {
            let a140 = {
                let a138 = v1;
                let a139 = 4294967296i64;
                int_mul(a138, a139)?
            };
            int_neg(a140)?
        };
        int_sub(a141, a142)?
    };
    let v5: i64 = {
        let a143 = v4;
        let a144 = 4294967296i64;
        let a145 = 0i64;
        int_quot(a143, a144, a145)?
    };
    let v6: i64 = {
        let m148 = {
            let a146 = v5;
            let a147 = 0i64;
            compare(a146, a147)
        };
        if match m148 {
            Ordering::Less => {
                true
            }
            Ordering::Equal => {
                false
            }
            Ordering::Greater => {
                false
            }
        } {
            0i64
        } else {
            v5
        }
    };
    let v7: i64 = {
        let m151 = {
            let a149 = 255i64;
            let a150 = v6;
            compare(a149, a150)
        };
        if match m151 {
            Ordering::Less => {
                true
            }
            Ordering::Equal => {
                false
            }
            Ordering::Greater => {
                false
            }
        } {
            255i64
        } else {
            v6
        }
    };
    Ok(v7)
}

pub fn f36() -> i64 {
    f21(f64(), 0u64)
}

pub fn f37(v0: List<i64>, v1: List<i64>) -> R<i64> {
    let (_, h152) = f65(Fn8::F66, (v1, 0i64), v0)?;
    Ok(h152)
}

pub fn f38() -> i64 {
    f21(f64(), 1u64)
}

pub fn f39() -> i64 {
    f21(f64(), 2u64)
}

pub fn f40() -> i64 {
    f21(f64(), 4u64)
}

pub fn f41() -> i64 {
    f21(f64(), 3u64)
}

pub fn f42(v0: Fn4, v1: List<i64>, v2: List<u64>) -> R<List<i64>> {
    let m153 = v2;
    match m153.uncons() {
        None => {
            Ok(v1)
        }
        Some((v3, v4)) => {
            f42(v0.clone(), {
                let c154: Fn4 = v0;
                c154.apply(v1, v3)?
            }, v4)
        }
    }
}

pub fn f43(v0: Bytes, v1: u64, v2: List<i64>, v3: u64) -> R<List<i64>> {
    Ok(List::cons(f67(v0, {
        let a157 = {
            let a155 = v1;
            let a156 = 28u64;
            nat_mul(a155, a156)?
        };
        let a158 = v3;
        nat_add(a157, a158)?
    })?, v2))
}

pub fn f44(v0: Fn5, v1: List<i64>, v2: List<List<List<i64>>>) -> List<i64> {
    let m159 = v2;
    match m159.uncons() {
        None => {
            v1
        }
        Some((v3, v4)) => {
            f44(v0.clone(), {
                let c160: Fn5 = v0;
                c160.apply(v1, v3)
            }, v4)
        }
    }
}

pub fn f45(v0: List<i64>, v1: List<List<i64>>) -> List<i64> {
    f68(v0, v1)
}

pub fn f46(v0: List<List<List<i64>>>) -> R<List<List<List<i64>>>> {
    f47(Fn6::F69, List::<List<List<i64>>>::nil(), v0)
}

pub fn f47(v0: Fn6, v1: List<List<List<i64>>>, v2: List<List<List<i64>>>) -> R<List<List<List<i64>>>> {
    let m161 = v2;
    match m161.uncons() {
        None => {
            Ok(v1)
        }
        Some((v3, v4)) => {
            f47(v0.clone(), {
                let c162: Fn6 = v0;
                c162.apply(v1, v3)?
            }, v4)
        }
    }
}

pub fn f48(v0: List<List<List<i64>>>, v1: List<List<i64>>) -> R<List<List<List<i64>>>> {
    Ok(List::cons(f70(v1)?, v0))
}

pub fn f49() -> List<List<List<List<i64>>>> {
    List::cons(List::cons(List::cons(List::cons(-14i64, List::cons(-26i64, List::cons(-2i64, List::cons(-36i64, List::cons(31i64, List::<i64>::nil()))))), List::cons(List::cons(24i64, List::cons(-47i64, List::cons(-76i64, List::cons(-5i64, List::cons(19i64, List::<i64>::nil()))))), List::cons(List::cons(11i64, List::cons(-3i64, List::cons(-18i64, List::cons(-42i64, List::cons(-32i64, List::<i64>::nil()))))), List::cons(List::cons(20i64, List::cons(0i64, List::cons(23i64, List::cons(-51i64, List::cons(-33i64, List::<i64>::nil()))))), List::cons(List::cons(1i64, List::cons(71i64, List::cons(54i64, List::cons(-15i64, List::cons(-36i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-119i64, List::cons(-76i64, List::cons(-73i64, List::cons(23i64, List::cons(25i64, List::<i64>::nil()))))), List::cons(List::cons(3i64, List::cons(-64i64, List::cons(-18i64, List::cons(6i64, List::cons(-14i64, List::<i64>::nil()))))), List::cons(List::cons(48i64, List::cons(66i64, List::cons(-5i64, List::cons(-38i64, List::cons(-12i64, List::<i64>::nil()))))), List::cons(List::cons(-19i64, List::cons(52i64, List::cons(51i64, List::cons(-41i64, List::cons(8i64, List::<i64>::nil()))))), List::cons(List::cons(-39i64, List::cons(32i64, List::cons(30i64, List::cons(14i64, List::cons(16i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-44i64, List::cons(-38i64, List::cons(22i64, List::cons(33i64, List::cons(-38i64, List::<i64>::nil()))))), List::cons(List::cons(-35i64, List::cons(-58i64, List::cons(-40i64, List::cons(-17i64, List::cons(-24i64, List::<i64>::nil()))))), List::cons(List::cons(-53i64, List::cons(-62i64, List::cons(-61i64, List::cons(-24i64, List::cons(18i64, List::<i64>::nil()))))), List::cons(List::cons(-73i64, List::cons(-29i64, List::cons(-94i64, List::cons(-38i64, List::cons(-9i64, List::<i64>::nil()))))), List::cons(List::cons(8i64, List::cons(25i64, List::cons(-31i64, List::cons(-53i64, List::cons(-63i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(29i64, List::cons(-13i64, List::cons(-7i64, List::cons(-6i64, List::cons(8i64, List::<i64>::nil()))))), List::cons(List::cons(13i64, List::cons(0i64, List::cons(29i64, List::cons(-25i64, List::cons(-15i64, List::<i64>::nil()))))), List::cons(List::cons(22i64, List::cons(7i64, List::cons(-72i64, List::cons(-80i64, List::cons(-59i64, List::<i64>::nil()))))), List::cons(List::cons(5i64, List::cons(19i64, List::cons(-10i64, List::cons(-86i64, List::cons(-36i64, List::<i64>::nil()))))), List::cons(List::cons(-3i64, List::cons(33i64, List::cons(48i64, List::cons(-19i64, List::cons(15i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(49i64, List::cons(-18i64, List::cons(-39i64, List::cons(-87i64, List::cons(-29i64, List::<i64>::nil()))))), List::cons(List::cons(26i64, List::cons(84i64, List::cons(17i64, List::cons(-63i64, List::cons(-61i64, List::<i64>::nil()))))), List::cons(List::cons(-24i64, List::cons(10i64, List::cons(34i64, List::cons(-40i64, List::cons(-40i64, List::<i64>::nil()))))), List::cons(List::cons(-119i64, List::cons(-10i64, List::cons(15i64, List::cons(7i64, List::cons(-5i64, List::<i64>::nil()))))), List::cons(List::cons(-34i64, List::cons(-46i64, List::cons(-27i64, List::cons(10i64, List::cons(4i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(40i64, List::cons(6i64, List::cons(-95i64, List::cons(-63i64, List::cons(-46i64, List::<i64>::nil()))))), List::cons(List::cons(47i64, List::cons(127i64, List::cons(75i64, List::cons(-35i64, List::cons(-61i64, List::<i64>::nil()))))), List::cons(List::cons(-42i64, List::cons(-21i64, List::cons(72i64, List::cons(29i64, List::cons(18i64, List::<i64>::nil()))))), List::cons(List::cons(-118i64, List::cons(-87i64, List::cons(-10i64, List::cons(13i64, List::cons(32i64, List::<i64>::nil()))))), List::cons(List::cons(-15i64, List::cons(-38i64, List::cons(-75i64, List::cons(-60i64, List::cons(-9i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(15i64, List::cons(-48i64, List::cons(-81i64, List::cons(-12i64, List::cons(-69i64, List::<i64>::nil()))))), List::cons(List::cons(28i64, List::cons(55i64, List::cons(-28i64, List::cons(-76i64, List::cons(-42i64, List::<i64>::nil()))))), List::cons(List::cons(-24i64, List::cons(1i64, List::cons(15i64, List::cons(13i64, List::cons(-45i64, List::<i64>::nil()))))), List::cons(List::cons(7i64, List::cons(18i64, List::cons(69i64, List::cons(17i64, List::cons(4i64, List::<i64>::nil()))))), List::cons(List::cons(-27i64, List::cons(-30i64, List::cons(64i64, List::cons(57i64, List::cons(-25i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-70i64, List::cons(41i64, List::cons(15i64, List::cons(-7i64, List::cons(28i64, List::<i64>::nil()))))), List::cons(List::cons(-16i64, List::cons(-21i64, List::cons(-16i64, List::cons(14i64, List::cons(24i64, List::<i64>::nil()))))), List::cons(List::cons(17i64, List::cons(-43i64, List::cons(-39i64, List::cons(6i64, List::cons(8i64, List::<i64>::nil()))))), List::cons(List::cons(-29i64, List::cons(8i64, List::cons(-36i64, List::cons(-64i64, List::cons(-76i64, List::<i64>::nil()))))), List::cons(List::cons(37i64, List::cons(22i64, List::cons(-35i64, List::cons(-41i64, List::cons(-49i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil())))))))), List::cons(List::cons(List::cons(List::cons(23i64, List::cons(66i64, List::cons(-6i64, List::cons(-4i64, List::cons(-49i64, List::<i64>::nil()))))), List::cons(List::cons(-12i64, List::cons(-4i64, List::cons(-11i64, List::cons(42i64, List::cons(78i64, List::<i64>::nil()))))), List::cons(List::cons(13i64, List::cons(-121i64, List::cons(-24i64, List::cons(46i64, List::cons(63i64, List::<i64>::nil()))))), List::cons(List::cons(-43i64, List::cons(-59i64, List::cons(-13i64, List::cons(28i64, List::cons(67i64, List::<i64>::nil()))))), List::cons(List::cons(-26i64, List::cons(-62i64, List::cons(-41i64, List::cons(-31i64, List::cons(-1i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-113i64, List::cons(-95i64, List::cons(-127i64, List::cons(-10i64, List::cons(-44i64, List::<i64>::nil()))))), List::cons(List::cons(-26i64, List::cons(-20i64, List::cons(-82i64, List::cons(-18i64, List::cons(-32i64, List::<i64>::nil()))))), List::cons(List::cons(11i64, List::cons(42i64, List::cons(-12i64, List::cons(-29i64, List::cons(2i64, List::<i64>::nil()))))), List::cons(List::cons(6i64, List::cons(71i64, List::cons(-3i64, List::cons(-53i64, List::cons(-56i64, List::<i64>::nil()))))), List::cons(List::cons(7i64, List::cons(-47i64, List::cons(-4i64, List::cons(-10i64, List::cons(-26i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-68i64, List::cons(-78i64, List::cons(-57i64, List::cons(-17i64, List::cons(29i64, List::<i64>::nil()))))), List::cons(List::cons(-32i64, List::cons(-52i64, List::cons(-48i64, List::cons(1i64, List::cons(2i64, List::<i64>::nil()))))), List::cons(List::cons(-2i64, List::cons(64i64, List::cons(20i64, List::cons(51i64, List::cons(85i64, List::<i64>::nil()))))), List::cons(List::cons(-18i64, List::cons(27i64, List::cons(45i64, List::cons(42i64, List::cons(-3i64, List::<i64>::nil()))))), List::cons(List::cons(39i64, List::cons(-5i64, List::cons(-24i64, List::cons(-10i64, List::cons(-47i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(42i64, List::cons(9i64, List::cons(-22i64, List::cons(-48i64, List::cons(-25i64, List::<i64>::nil()))))), List::cons(List::cons(7i64, List::cons(2i64, List::cons(-43i64, List::cons(35i64, List::cons(73i64, List::<i64>::nil()))))), List::cons(List::cons(-21i64, List::cons(-39i64, List::cons(-7i64, List::cons(67i64, List::cons(74i64, List::<i64>::nil()))))), List::cons(List::cons(-70i64, List::cons(-3i64, List::cons(-60i64, List::cons(-27i64, List::cons(-55i64, List::<i64>::nil()))))), List::cons(List::cons(-61i64, List::cons(-95i64, List::cons(-34i64, List::cons(-37i64, List::cons(13i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(4i64, List::cons(-28i64, List::cons(-50i64, List::cons(-50i64, List::cons(-29i64, List::<i64>::nil()))))), List::cons(List::cons(31i64, List::cons(71i64, List::cons(103i64, List::cons(-49i64, List::cons(-9i64, List::<i64>::nil()))))), List::cons(List::cons(47i64, List::cons(10i64, List::cons(63i64, List::cons(33i64, List::cons(39i64, List::<i64>::nil()))))), List::cons(List::cons(8i64, List::cons(-57i64, List::cons(-66i64, List::cons(-47i64, List::cons(-60i64, List::<i64>::nil()))))), List::cons(List::cons(-31i64, List::cons(-21i64, List::cons(-56i64, List::cons(-69i64, List::cons(-19i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-65i64, List::cons(-27i64, List::cons(-41i64, List::cons(-37i64, List::cons(-46i64, List::<i64>::nil()))))), List::cons(List::cons(31i64, List::cons(37i64, List::cons(70i64, List::cons(-9i64, List::cons(14i64, List::<i64>::nil()))))), List::cons(List::cons(48i64, List::cons(28i64, List::cons(70i64, List::cons(-7i64, List::cons(59i64, List::<i64>::nil()))))), List::cons(List::cons(23i64, List::cons(15i64, List::cons(-12i64, List::cons(-14i64, List::cons(-70i64, List::<i64>::nil()))))), List::cons(List::cons(-11i64, List::cons(13i64, List::cons(-24i64, List::cons(-80i64, List::cons(-28i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-53i64, List::cons(-73i64, List::cons(32i64, List::cons(-10i64, List::cons(-1i64, List::<i64>::nil()))))), List::cons(List::cons(-15i64, List::cons(42i64, List::cons(67i64, List::cons(-15i64, List::cons(-13i64, List::<i64>::nil()))))), List::cons(List::cons(-20i64, List::cons(-21i64, List::cons(3i64, List::cons(75i64, List::cons(42i64, List::<i64>::nil()))))), List::cons(List::cons(-3i64, List::cons(-44i64, List::cons(34i64, List::cons(23i64, List::cons(-27i64, List::<i64>::nil()))))), List::cons(List::cons(15i64, List::cons(-14i64, List::cons(-58i64, List::cons(-51i64, List::cons(-16i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-26i64, List::cons(8i64, List::cons(-27i64, List::cons(-57i64, List::cons(2i64, List::<i64>::nil()))))), List::cons(List::cons(-42i64, List::cons(-74i64, List::cons(-85i64, List::cons(-33i64, List::cons(0i64, List::<i64>::nil()))))), List::cons(List::cons(-24i64, List::cons(35i64, List::cons(30i64, List::cons(-12i64, List::cons(39i64, List::<i64>::nil()))))), List::cons(List::cons(27i64, List::cons(83i64, List::cons(62i64, List::cons(64i64, List::cons(12i64, List::<i64>::nil()))))), List::cons(List::cons(-46i64, List::cons(-29i64, List::cons(-33i64, List::cons(-60i64, List::cons(-83i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil())))))))), List::cons(List::cons(List::cons(List::cons(-39i64, List::cons(-7i64, List::cons(24i64, List::cons(-2i64, List::cons(-4i64, List::<i64>::nil()))))), List::cons(List::cons(-45i64, List::cons(-15i64, List::cons(-20i64, List::cons(-55i64, List::cons(27i64, List::<i64>::nil()))))), List::cons(List::cons(-127i64, List::cons(-77i64, List::cons(-36i64, List::cons(3i64, List::cons(-8i64, List::<i64>::nil()))))), List::cons(List::cons(-15i64, List::cons(-57i64, List::cons(-42i64, List::cons(-29i64, List::cons(-23i64, List::<i64>::nil()))))), List::cons(List::cons(16i64, List::cons(-59i64, List::cons(-4i64, List::cons(7i64, List::cons(52i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-43i64, List::cons(78i64, List::cons(55i64, List::cons(-59i64, List::cons(-43i64, List::<i64>::nil()))))), List::cons(List::cons(-54i64, List::cons(50i64, List::cons(42i64, List::cons(15i64, List::cons(-11i64, List::<i64>::nil()))))), List::cons(List::cons(-67i64, List::cons(17i64, List::cons(73i64, List::cons(9i64, List::cons(42i64, List::<i64>::nil()))))), List::cons(List::cons(-81i64, List::cons(-65i64, List::cons(-18i64, List::cons(-49i64, List::cons(18i64, List::<i64>::nil()))))), List::cons(List::cons(5i64, List::cons(-18i64, List::cons(-14i64, List::cons(-7i64, List::cons(-47i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(23i64, List::cons(-3i64, List::cons(-46i64, List::cons(-8i64, List::cons(-20i64, List::<i64>::nil()))))), List::cons(List::cons(30i64, List::cons(8i64, List::cons(-38i64, List::cons(-31i64, List::cons(90i64, List::<i64>::nil()))))), List::cons(List::cons(76i64, List::cons(40i64, List::cons(29i64, List::cons(43i64, List::cons(54i64, List::<i64>::nil()))))), List::cons(List::cons(-6i64, List::cons(-13i64, List::cons(-5i64, List::cons(27i64, List::cons(-29i64, List::<i64>::nil()))))), List::cons(List::cons(26i64, List::cons(-16i64, List::cons(-29i64, List::cons(-33i64, List::cons(-18i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(20i64, List::cons(5i64, List::cons(27i64, List::cons(21i64, List::cons(39i64, List::<i64>::nil()))))), List::cons(List::cons(-74i64, List::cons(-48i64, List::cons(-18i64, List::cons(12i64, List::cons(50i64, List::<i64>::nil()))))), List::cons(List::cons(-55i64, List::cons(-71i64, List::cons(-98i64, List::cons(-32i64, List::cons(-17i64, List::<i64>::nil()))))), List::cons(List::cons(63i64, List::cons(24i64, List::cons(-15i64, List::cons(-34i64, List::cons(-63i64, List::<i64>::nil()))))), List::cons(List::cons(66i64, List::cons(11i64, List::cons(16i64, List::cons(45i64, List::cons(44i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(22i64, List::cons(6i64, List::cons(-19i64, List::cons(19i64, List::cons(-12i64, List::<i64>::nil()))))), List::cons(List::cons(-21i64, List::cons(-16i64, List::cons(-31i64, List::cons(-21i64, List::cons(4i64, List::<i64>::nil()))))), List::cons(List::cons(9i64, List::cons(-41i64, List::cons(-61i64, List::cons(-92i64, List::cons(-45i64, List::<i64>::nil()))))), List::cons(List::cons(35i64, List::cons(26i64, List::cons(-25i64, List::cons(-39i64, List::cons(-100i64, List::<i64>::nil()))))), List::cons(List::cons(20i64, List::cons(29i64, List::cons(12i64, List::cons(52i64, List::cons(26i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-1i64, List::cons(1i64, List::cons(-47i64, List::cons(23i64, List::cons(-22i64, List::<i64>::nil()))))), List::cons(List::cons(-12i64, List::cons(-40i64, List::cons(-10i64, List::cons(-12i64, List::cons(-13i64, List::<i64>::nil()))))), List::cons(List::cons(16i64, List::cons(5i64, List::cons(-32i64, List::cons(-48i64, List::cons(-67i64, List::<i64>::nil()))))), List::cons(List::cons(24i64, List::cons(57i64, List::cons(24i64, List::cons(-10i64, List::cons(-59i64, List::<i64>::nil()))))), List::cons(List::cons(-35i64, List::cons(10i64, List::cons(54i64, List::cons(17i64, List::cons(76i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-17i64, List::cons(-15i64, List::cons(-3i64, List::cons(56i64, List::cons(30i64, List::<i64>::nil()))))), List::cons(List::cons(-16i64, List::cons(-19i64, List::cons(1i64, List::cons(-16i64, List::cons(-75i64, List::<i64>::nil()))))), List::cons(List::cons(-35i64, List::cons(-55i64, List::cons(-40i64, List::cons(-68i64, List::cons(-71i64, List::<i64>::nil()))))), List::cons(List::cons(-54i64, List::cons(-18i64, List::cons(-49i64, List::cons(-68i64, List::cons(-66i64, List::<i64>::nil()))))), List::cons(List::cons(-46i64, List::cons(56i64, List::cons(5i64, List::cons(0i64, List::cons(3i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(16i64, List::cons(-8i64, List::cons(-52i64, List::cons(-51i64, List::cons(-52i64, List::<i64>::nil()))))), List::cons(List::cons(-5i64, List::cons(13i64, List::cons(1i64, List::cons(-13i64, List::cons(59i64, List::<i64>::nil()))))), List::cons(List::cons(-16i64, List::cons(13i64, List::cons(67i64, List::cons(86i64, List::cons(51i64, List::<i64>::nil()))))), List::cons(List::cons(-104i64, List::cons(-61i64, List::cons(-26i64, List::cons(-11i64, List::cons(7i64, List::<i64>::nil()))))), List::cons(List::cons(22i64, List::cons(-56i64, List::cons(-57i64, List::cons(2i64, List::cons(-5i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil())))))))), List::cons(List::cons(List::cons(List::cons(105i64, List::cons(16i64, List::cons(-32i64, List::cons(-18i64, List::cons(-50i64, List::<i64>::nil()))))), List::cons(List::cons(-3i64, List::cons(-65i64, List::cons(27i64, List::cons(-31i64, List::cons(-42i64, List::<i64>::nil()))))), List::cons(List::cons(-16i64, List::cons(-50i64, List::cons(25i64, List::cons(18i64, List::cons(-18i64, List::<i64>::nil()))))), List::cons(List::cons(-4i64, List::cons(-39i64, List::cons(48i64, List::cons(-11i64, List::cons(-6i64, List::<i64>::nil()))))), List::cons(List::cons(-52i64, List::cons(-103i64, List::cons(-70i64, List::cons(6i64, List::cons(-71i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(22i64, List::cons(39i64, List::cons(39i64, List::cons(-16i64, List::cons(0i64, List::<i64>::nil()))))), List::cons(List::cons(30i64, List::cons(81i64, List::cons(13i64, List::cons(-29i64, List::cons(83i64, List::<i64>::nil()))))), List::cons(List::cons(4i64, List::cons(20i64, List::cons(91i64, List::cons(-65i64, List::cons(-25i64, List::<i64>::nil()))))), List::cons(List::cons(-47i64, List::cons(5i64, List::cons(8i64, List::cons(-90i64, List::cons(-48i64, List::<i64>::nil()))))), List::cons(List::cons(-36i64, List::cons(-52i64, List::cons(-15i64, List::cons(-28i64, List::cons(-107i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-26i64, List::cons(-46i64, List::cons(19i64, List::cons(-20i64, List::cons(33i64, List::<i64>::nil()))))), List::cons(List::cons(10i64, List::cons(-12i64, List::cons(-13i64, List::cons(-46i64, List::cons(14i64, List::<i64>::nil()))))), List::cons(List::cons(-60i64, List::cons(-30i64, List::cons(-8i64, List::cons(-3i64, List::cons(-57i64, List::<i64>::nil()))))), List::cons(List::cons(10i64, List::cons(17i64, List::cons(18i64, List::cons(28i64, List::cons(-23i64, List::<i64>::nil()))))), List::cons(List::cons(13i64, List::cons(6i64, List::cons(7i64, List::cons(27i64, List::cons(23i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(23i64, List::cons(61i64, List::cons(41i64, List::cons(-25i64, List::cons(-29i64, List::<i64>::nil()))))), List::cons(List::cons(-7i64, List::cons(20i64, List::cons(5i64, List::cons(-62i64, List::cons(-122i64, List::<i64>::nil()))))), List::cons(List::cons(17i64, List::cons(46i64, List::cons(-5i64, List::cons(-15i64, List::cons(-58i64, List::<i64>::nil()))))), List::cons(List::cons(-12i64, List::cons(-30i64, List::cons(-75i64, List::cons(-34i64, List::cons(-39i64, List::<i64>::nil()))))), List::cons(List::cons(-60i64, List::cons(-66i64, List::cons(-28i64, List::cons(-35i64, List::cons(-62i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-36i64, List::cons(-19i64, List::cons(31i64, List::cons(-1i64, List::cons(30i64, List::<i64>::nil()))))), List::cons(List::cons(-86i64, List::cons(-86i64, List::cons(33i64, List::cons(52i64, List::cons(-48i64, List::<i64>::nil()))))), List::cons(List::cons(-31i64, List::cons(-18i64, List::cons(-53i64, List::cons(34i64, List::cons(-40i64, List::<i64>::nil()))))), List::cons(List::cons(-71i64, List::cons(-30i64, List::cons(-32i64, List::cons(-32i64, List::cons(-28i64, List::<i64>::nil()))))), List::cons(List::cons(-31i64, List::cons(-39i64, List::cons(-22i64, List::cons(-10i64, List::cons(-63i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-43i64, List::cons(-70i64, List::cons(-26i64, List::cons(7i64, List::cons(50i64, List::<i64>::nil()))))), List::cons(List::cons(-49i64, List::cons(-74i64, List::cons(-58i64, List::cons(26i64, List::cons(2i64, List::<i64>::nil()))))), List::cons(List::cons(-59i64, List::cons(-11i64, List::cons(-44i64, List::cons(-14i64, List::cons(11i64, List::<i64>::nil()))))), List::cons(List::cons(-88i64, List::cons(-52i64, List::cons(-58i64, List::cons(-72i64, List::cons(-14i64, List::<i64>::nil()))))), List::cons(List::cons(-64i64, List::cons(-19i64, List::cons(-8i64, List::cons(-28i64, List::cons(-6i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-67i64, List::cons(-53i64, List::cons(-16i64, List::cons(50i64, List::cons(-33i64, List::<i64>::nil()))))), List::cons(List::cons(-57i64, List::cons(-25i64, List::cons(8i64, List::cons(98i64, List::cons(-6i64, List::<i64>::nil()))))), List::cons(List::cons(-12i64, List::cons(-5i64, List::cons(22i64, List::cons(127i64, List::cons(60i64, List::<i64>::nil()))))), List::cons(List::cons(-47i64, List::cons(-25i64, List::cons(59i64, List::cons(80i64, List::cons(2i64, List::<i64>::nil()))))), List::cons(List::cons(-20i64, List::cons(-7i64, List::cons(9i64, List::cons(-34i64, List::cons(-24i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(2i64, List::cons(-9i64, List::cons(63i64, List::cons(32i64, List::cons(19i64, List::<i64>::nil()))))), List::cons(List::cons(-33i64, List::cons(24i64, List::cons(0i64, List::cons(60i64, List::cons(18i64, List::<i64>::nil()))))), List::cons(List::cons(-65i64, List::cons(-34i64, List::cons(47i64, List::cons(-12i64, List::cons(-17i64, List::<i64>::nil()))))), List::cons(List::cons(-18i64, List::cons(53i64, List::cons(54i64, List::cons(-22i64, List::cons(-48i64, List::<i64>::nil()))))), List::cons(List::cons(30i64, List::cons(-27i64, List::cons(12i64, List::cons(-38i64, List::cons(-55i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil())))))))), List::cons(List::cons(List::cons(List::cons(-11i64, List::cons(-37i64, List::cons(-20i64, List::cons(-4i64, List::cons(-57i64, List::<i64>::nil()))))), List::cons(List::cons(54i64, List::cons(70i64, List::cons(34i64, List::cons(26i64, List::cons(-37i64, List::<i64>::nil()))))), List::cons(List::cons(53i64, List::cons(65i64, List::cons(17i64, List::cons(79i64, List::cons(-14i64, List::<i64>::nil()))))), List::cons(List::cons(-23i64, List::cons(-9i64, List::cons(117i64, List::cons(36i64, List::cons(5i64, List::<i64>::nil()))))), List::cons(List::cons(-55i64, List::cons(-5i64, List::cons(32i64, List::cons(2i64, List::cons(-18i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-1i64, List::cons(-43i64, List::cons(-20i64, List::cons(-60i64, List::cons(-15i64, List::<i64>::nil()))))), List::cons(List::cons(4i64, List::cons(-16i64, List::cons(30i64, List::cons(56i64, List::cons(83i64, List::<i64>::nil()))))), List::cons(List::cons(45i64, List::cons(-33i64, List::cons(-2i64, List::cons(4i64, List::cons(9i64, List::<i64>::nil()))))), List::cons(List::cons(-13i64, List::cons(-45i64, List::cons(-81i64, List::cons(-62i64, List::cons(-71i64, List::<i64>::nil()))))), List::cons(List::cons(59i64, List::cons(41i64, List::cons(-47i64, List::cons(-67i64, List::cons(-90i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-22i64, List::cons(-16i64, List::cons(22i64, List::cons(51i64, List::cons(50i64, List::<i64>::nil()))))), List::cons(List::cons(-14i64, List::cons(-51i64, List::cons(-48i64, List::cons(-26i64, List::cons(-3i64, List::<i64>::nil()))))), List::cons(List::cons(-20i64, List::cons(-36i64, List::cons(-89i64, List::cons(-44i64, List::cons(-21i64, List::<i64>::nil()))))), List::cons(List::cons(-22i64, List::cons(14i64, List::cons(22i64, List::cons(4i64, List::cons(0i64, List::<i64>::nil()))))), List::cons(List::cons(82i64, List::cons(127i64, List::cons(102i64, List::cons(16i64, List::cons(-28i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-28i64, List::cons(-25i64, List::cons(-24i64, List::cons(-47i64, List::cons(-21i64, List::<i64>::nil()))))), List::cons(List::cons(-49i64, List::cons(5i64, List::cons(5i64, List::cons(-32i64, List::cons(-46i64, List::<i64>::nil()))))), List::cons(List::cons(-41i64, List::cons(-4i64, List::cons(-8i64, List::cons(37i64, List::cons(-61i64, List::<i64>::nil()))))), List::cons(List::cons(-8i64, List::cons(8i64, List::cons(-18i64, List::cons(1i64, List::cons(-29i64, List::<i64>::nil()))))), List::cons(List::cons(-40i64, List::cons(-72i64, List::cons(-33i64, List::cons(-82i64, List::cons(-21i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(8i64, List::cons(90i64, List::cons(74i64, List::cons(35i64, List::cons(29i64, List::<i64>::nil()))))), List::cons(List::cons(-1i64, List::cons(21i64, List::cons(9i64, List::cons(-10i64, List::cons(-26i64, List::<i64>::nil()))))), List::cons(List::cons(-12i64, List::cons(11i64, List::cons(-18i64, List::cons(26i64, List::cons(-21i64, List::<i64>::nil()))))), List::cons(List::cons(12i64, List::cons(35i64, List::cons(-1i64, List::cons(-16i64, List::cons(-20i64, List::<i64>::nil()))))), List::cons(List::cons(-12i64, List::cons(-5i64, List::cons(-66i64, List::cons(-81i64, List::cons(-72i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-11i64, List::cons(30i64, List::cons(56i64, List::cons(49i64, List::cons(75i64, List::<i64>::nil()))))), List::cons(List::cons(13i64, List::cons(60i64, List::cons(23i64, List::cons(69i64, List::cons(-18i64, List::<i64>::nil()))))), List::cons(List::cons(-5i64, List::cons(-30i64, List::cons(-29i64, List::cons(2i64, List::cons(-38i64, List::<i64>::nil()))))), List::cons(List::cons(30i64, List::cons(49i64, List::cons(-29i64, List::cons(-81i64, List::cons(-62i64, List::<i64>::nil()))))), List::cons(List::cons(4i64, List::cons(12i64, List::cons(-29i64, List::cons(-48i64, List::cons(-22i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-33i64, List::cons(-10i64, List::cons(11i64, List::cons(-28i64, List::cons(-26i64, List::<i64>::nil()))))), List::cons(List::cons(-39i64, List::cons(-46i64, List::cons(2i64, List::cons(16i64, List::cons(6i64, List::<i64>::nil()))))), List::cons(List::cons(34i64, List::cons(-32i64, List::cons(11i64, List::cons(20i64, List::cons(8i64, List::<i64>::nil()))))), List::cons(List::cons(4i64, List::cons(-34i64, List::cons(-71i64, List::cons(14i64, List::cons(25i64, List::<i64>::nil()))))), List::cons(List::cons(-69i64, List::cons(-31i64, List::cons(-33i64, List::cons(-30i64, List::cons(-9i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(58i64, List::cons(14i64, List::cons(-6i64, List::cons(-41i64, List::cons(-52i64, List::<i64>::nil()))))), List::cons(List::cons(55i64, List::cons(33i64, List::cons(-5i64, List::cons(-42i64, List::cons(-67i64, List::<i64>::nil()))))), List::cons(List::cons(-36i64, List::cons(-28i64, List::cons(-39i64, List::cons(-18i64, List::cons(-29i64, List::<i64>::nil()))))), List::cons(List::cons(-50i64, List::cons(-21i64, List::cons(17i64, List::cons(25i64, List::cons(-39i64, List::<i64>::nil()))))), List::cons(List::cons(24i64, List::cons(100i64, List::cons(49i64, List::cons(-12i64, List::cons(-39i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil())))))))), List::cons(List::cons(List::cons(List::cons(74i64, List::cons(91i64, List::cons(33i64, List::cons(84i64, List::cons(117i64, List::<i64>::nil()))))), List::cons(List::cons(-18i64, List::cons(27i64, List::cons(-29i64, List::cons(-21i64, List::cons(5i64, List::<i64>::nil()))))), List::cons(List::cons(-35i64, List::cons(20i64, List::cons(-91i64, List::cons(-127i64, List::cons(-60i64, List::<i64>::nil()))))), List::cons(List::cons(-12i64, List::cons(-58i64, List::cons(-57i64, List::cons(-64i64, List::cons(-92i64, List::<i64>::nil()))))), List::cons(List::cons(-13i64, List::cons(-44i64, List::cons(-50i64, List::cons(-25i64, List::cons(-93i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(30i64, List::cons(83i64, List::cons(26i64, List::cons(-30i64, List::cons(-33i64, List::<i64>::nil()))))), List::cons(List::cons(32i64, List::cons(46i64, List::cons(54i64, List::cons(-5i64, List::cons(-26i64, List::<i64>::nil()))))), List::cons(List::cons(2i64, List::cons(-25i64, List::cons(63i64, List::cons(90i64, List::cons(-39i64, List::<i64>::nil()))))), List::cons(List::cons(17i64, List::cons(3i64, List::cons(16i64, List::cons(63i64, List::cons(12i64, List::<i64>::nil()))))), List::cons(List::cons(-1i64, List::cons(0i64, List::cons(80i64, List::cons(8i64, List::cons(-10i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-58i64, List::cons(-61i64, List::cons(-117i64, List::cons(-56i64, List::cons(-61i64, List::<i64>::nil()))))), List::cons(List::cons(-37i64, List::cons(-63i64, List::cons(-72i64, List::cons(-19i64, List::cons(-37i64, List::<i64>::nil()))))), List::cons(List::cons(8i64, List::cons(19i64, List::cons(-6i64, List::cons(-24i64, List::cons(47i64, List::<i64>::nil()))))), List::cons(List::cons(39i64, List::cons(15i64, List::cons(-43i64, List::cons(-50i64, List::cons(58i64, List::<i64>::nil()))))), List::cons(List::cons(-16i64, List::cons(-44i64, List::cons(-72i64, List::cons(13i64, List::cons(47i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(88i64, List::cons(3i64, List::cons(17i64, List::cons(-43i64, List::cons(8i64, List::<i64>::nil()))))), List::cons(List::cons(-13i64, List::cons(-28i64, List::cons(26i64, List::cons(-35i64, List::cons(-16i64, List::<i64>::nil()))))), List::cons(List::cons(-28i64, List::cons(-8i64, List::cons(-28i64, List::cons(-2i64, List::cons(-51i64, List::<i64>::nil()))))), List::cons(List::cons(-18i64, List::cons(-8i64, List::cons(-77i64, List::cons(-51i64, List::cons(-105i64, List::<i64>::nil()))))), List::cons(List::cons(4i64, List::cons(-62i64, List::cons(-5i64, List::cons(-44i64, List::cons(-47i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-37i64, List::cons(-77i64, List::cons(-12i64, List::cons(45i64, List::cons(32i64, List::<i64>::nil()))))), List::cons(List::cons(23i64, List::cons(-56i64, List::cons(-101i64, List::cons(39i64, List::cons(62i64, List::<i64>::nil()))))), List::cons(List::cons(9i64, List::cons(-3i64, List::cons(-65i64, List::cons(-4i64, List::cons(48i64, List::<i64>::nil()))))), List::cons(List::cons(44i64, List::cons(18i64, List::cons(-44i64, List::cons(4i64, List::cons(10i64, List::<i64>::nil()))))), List::cons(List::cons(-7i64, List::cons(-7i64, List::cons(-3i64, List::cons(12i64, List::cons(2i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(30i64, List::cons(-47i64, List::cons(-98i64, List::cons(-23i64, List::cons(-19i64, List::<i64>::nil()))))), List::cons(List::cons(-2i64, List::cons(-47i64, List::cons(-106i64, List::cons(-62i64, List::cons(35i64, List::<i64>::nil()))))), List::cons(List::cons(20i64, List::cons(-33i64, List::cons(-74i64, List::cons(-32i64, List::cons(95i64, List::<i64>::nil()))))), List::cons(List::cons(27i64, List::cons(26i64, List::cons(0i64, List::cons(13i64, List::cons(31i64, List::<i64>::nil()))))), List::cons(List::cons(-20i64, List::cons(20i64, List::cons(11i64, List::cons(-5i64, List::cons(-11i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-44i64, List::cons(35i64, List::cons(59i64, List::cons(-2i64, List::cons(5i64, List::<i64>::nil()))))), List::cons(List::cons(38i64, List::cons(19i64, List::cons(-41i64, List::cons(14i64, List::cons(45i64, List::<i64>::nil()))))), List::cons(List::cons(47i64, List::cons(-33i64, List::cons(-40i64, List::cons(-20i64, List::cons(35i64, List::<i64>::nil()))))), List::cons(List::cons(-28i64, List::cons(-9i64, List::cons(-20i64, List::cons(-15i64, List::cons(-61i64, List::<i64>::nil()))))), List::cons(List::cons(-45i64, List::cons(33i64, List::cons(0i64, List::cons(42i64, List::cons(-4i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(22i64, List::cons(-2i64, List::cons(-27i64, List::cons(-6i64, List::cons(35i64, List::<i64>::nil()))))), List::cons(List::cons(37i64, List::cons(22i64, List::cons(25i64, List::cons(15i64, List::cons(14i64, List::<i64>::nil()))))), List::cons(List::cons(-15i64, List::cons(11i64, List::cons(17i64, List::cons(23i64, List::cons(57i64, List::<i64>::nil()))))), List::cons(List::cons(52i64, List::cons(-18i64, List::cons(-4i64, List::cons(28i64, List::cons(57i64, List::<i64>::nil()))))), List::cons(List::cons(-27i64, List::cons(-45i64, List::cons(-65i64, List::cons(-19i64, List::cons(-22i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil())))))))), List::cons(List::cons(List::cons(List::cons(20i64, List::cons(11i64, List::cons(-16i64, List::cons(-56i64, List::cons(-40i64, List::<i64>::nil()))))), List::cons(List::cons(13i64, List::cons(4i64, List::cons(-36i64, List::cons(32i64, List::cons(24i64, List::<i64>::nil()))))), List::cons(List::cons(-22i64, List::cons(-19i64, List::cons(6i64, List::cons(74i64, List::cons(75i64, List::<i64>::nil()))))), List::cons(List::cons(-25i64, List::cons(-26i64, List::cons(50i64, List::cons(-4i64, List::cons(18i64, List::<i64>::nil()))))), List::cons(List::cons(-46i64, List::cons(4i64, List::cons(-2i64, List::cons(-104i64, List::cons(-94i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(6i64, List::cons(-30i64, List::cons(-51i64, List::cons(-63i64, List::cons(-76i64, List::<i64>::nil()))))), List::cons(List::cons(-25i64, List::cons(-77i64, List::cons(-54i64, List::cons(-17i64, List::cons(-24i64, List::<i64>::nil()))))), List::cons(List::cons(-34i64, List::cons(-20i64, List::cons(-4i64, List::cons(-34i64, List::cons(26i64, List::<i64>::nil()))))), List::cons(List::cons(-48i64, List::cons(30i64, List::cons(36i64, List::cons(-34i64, List::cons(5i64, List::<i64>::nil()))))), List::cons(List::cons(3i64, List::cons(4i64, List::cons(-32i64, List::cons(-45i64, List::cons(28i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-53i64, List::cons(-1i64, List::cons(-21i64, List::cons(-4i64, List::cons(56i64, List::<i64>::nil()))))), List::cons(List::cons(-51i64, List::cons(-43i64, List::cons(-17i64, List::cons(-33i64, List::cons(34i64, List::<i64>::nil()))))), List::cons(List::cons(-39i64, List::cons(-27i64, List::cons(-17i64, List::cons(25i64, List::cons(114i64, List::<i64>::nil()))))), List::cons(List::cons(-30i64, List::cons(-49i64, List::cons(-43i64, List::cons(-34i64, List::cons(-15i64, List::<i64>::nil()))))), List::cons(List::cons(-53i64, List::cons(-78i64, List::cons(-36i64, List::cons(3i64, List::cons(42i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(15i64, List::cons(-56i64, List::cons(-37i64, List::cons(-13i64, List::cons(17i64, List::<i64>::nil()))))), List::cons(List::cons(-6i64, List::cons(-54i64, List::cons(-34i64, List::cons(54i64, List::cons(77i64, List::<i64>::nil()))))), List::cons(List::cons(-102i64, List::cons(0i64, List::cons(42i64, List::cons(39i64, List::cons(-44i64, List::<i64>::nil()))))), List::cons(List::cons(-58i64, List::cons(-32i64, List::cons(8i64, List::cons(-22i64, List::cons(-62i64, List::<i64>::nil()))))), List::cons(List::cons(-54i64, List::cons(-29i64, List::cons(-20i64, List::cons(-127i64, List::cons(-67i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(24i64, List::cons(-16i64, List::cons(-44i64, List::cons(-25i64, List::cons(61i64, List::<i64>::nil()))))), List::cons(List::cons(8i64, List::cons(-35i64, List::cons(-30i64, List::cons(16i64, List::cons(50i64, List::<i64>::nil()))))), List::cons(List::cons(5i64, List::cons(-29i64, List::cons(-23i64, List::cons(-1i64, List::cons(-11i64, List::<i64>::nil()))))), List::cons(List::cons(7i64, List::cons(-17i64, List::cons(-28i64, List::cons(19i64, List::cons(0i64, List::<i64>::nil()))))), List::cons(List::cons(36i64, List::cons(-21i64, List::cons(13i64, List::cons(-19i64, List::cons(9i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(17i64, List::cons(5i64, List::cons(-2i64, List::cons(-1i64, List::cons(38i64, List::<i64>::nil()))))), List::cons(List::cons(-23i64, List::cons(-4i64, List::cons(-1i64, List::cons(11i64, List::cons(34i64, List::<i64>::nil()))))), List::cons(List::cons(-50i64, List::cons(10i64, List::cons(20i64, List::cons(-10i64, List::cons(53i64, List::<i64>::nil()))))), List::cons(List::cons(-14i64, List::cons(-34i64, List::cons(-44i64, List::cons(21i64, List::cons(9i64, List::<i64>::nil()))))), List::cons(List::cons(34i64, List::cons(15i64, List::cons(-38i64, List::cons(-13i64, List::cons(6i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(25i64, List::cons(23i64, List::cons(-48i64, List::cons(-85i64, List::cons(-45i64, List::<i64>::nil()))))), List::cons(List::cons(65i64, List::cons(29i64, List::cons(-32i64, List::cons(-69i64, List::cons(-14i64, List::<i64>::nil()))))), List::cons(List::cons(87i64, List::cons(-3i64, List::cons(-35i64, List::cons(-6i64, List::cons(-41i64, List::<i64>::nil()))))), List::cons(List::cons(34i64, List::cons(-43i64, List::cons(9i64, List::cons(94i64, List::cons(-1i64, List::<i64>::nil()))))), List::cons(List::cons(-23i64, List::cons(-43i64, List::cons(107i64, List::cons(22i64, List::cons(-1i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-45i64, List::cons(-46i64, List::cons(-66i64, List::cons(-50i64, List::cons(-56i64, List::<i64>::nil()))))), List::cons(List::cons(8i64, List::cons(-79i64, List::cons(-38i64, List::cons(-42i64, List::cons(-6i64, List::<i64>::nil()))))), List::cons(List::cons(-41i64, List::cons(-89i64, List::cons(4i64, List::cons(66i64, List::cons(108i64, List::<i64>::nil()))))), List::cons(List::cons(-46i64, List::cons(-42i64, List::cons(39i64, List::cons(13i64, List::cons(11i64, List::<i64>::nil()))))), List::cons(List::cons(-84i64, List::cons(-5i64, List::cons(22i64, List::cons(3i64, List::cons(-11i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil())))))))), List::cons(List::cons(List::cons(List::cons(9i64, List::cons(33i64, List::cons(-3i64, List::cons(-8i64, List::cons(-39i64, List::<i64>::nil()))))), List::cons(List::cons(17i64, List::cons(31i64, List::cons(-31i64, List::cons(-37i64, List::cons(-85i64, List::<i64>::nil()))))), List::cons(List::cons(5i64, List::cons(109i64, List::cons(12i64, List::cons(-60i64, List::cons(39i64, List::<i64>::nil()))))), List::cons(List::cons(51i64, List::cons(57i64, List::cons(-37i64, List::cons(81i64, List::cons(27i64, List::<i64>::nil()))))), List::cons(List::cons(13i64, List::cons(-3i64, List::cons(88i64, List::cons(16i64, List::cons(48i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-42i64, List::cons(17i64, List::cons(-45i64, List::cons(-47i64, List::cons(93i64, List::<i64>::nil()))))), List::cons(List::cons(53i64, List::cons(5i64, List::cons(-65i64, List::cons(-44i64, List::cons(78i64, List::<i64>::nil()))))), List::cons(List::cons(47i64, List::cons(-89i64, List::cons(-28i64, List::cons(38i64, List::cons(124i64, List::<i64>::nil()))))), List::cons(List::cons(55i64, List::cons(-65i64, List::cons(27i64, List::cons(-15i64, List::cons(113i64, List::<i64>::nil()))))), List::cons(List::cons(-85i64, List::cons(-36i64, List::cons(-7i64, List::cons(46i64, List::cons(3i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-20i64, List::cons(-15i64, List::cons(7i64, List::cons(27i64, List::cons(-13i64, List::<i64>::nil()))))), List::cons(List::cons(-70i64, List::cons(-98i64, List::cons(21i64, List::cons(2i64, List::cons(-35i64, List::<i64>::nil()))))), List::cons(List::cons(-71i64, List::cons(-28i64, List::cons(20i64, List::cons(-13i64, List::cons(-92i64, List::<i64>::nil()))))), List::cons(List::cons(51i64, List::cons(25i64, List::cons(-12i64, List::cons(-78i64, List::cons(-72i64, List::<i64>::nil()))))), List::cons(List::cons(63i64, List::cons(-4i64, List::cons(-62i64, List::cons(6i64, List::cons(-13i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(26i64, List::cons(53i64, List::cons(81i64, List::cons(34i64, List::cons(9i64, List::<i64>::nil()))))), List::cons(List::cons(9i64, List::cons(17i64, List::cons(4i64, List::cons(-41i64, List::cons(47i64, List::<i64>::nil()))))), List::cons(List::cons(11i64, List::cons(29i64, List::cons(-63i64, List::cons(-92i64, List::cons(36i64, List::<i64>::nil()))))), List::cons(List::cons(-41i64, List::cons(-82i64, List::cons(-92i64, List::cons(71i64, List::cons(107i64, List::<i64>::nil()))))), List::cons(List::cons(-49i64, List::cons(-61i64, List::cons(-66i64, List::cons(21i64, List::cons(30i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-44i64, List::cons(42i64, List::cons(88i64, List::cons(122i64, List::cons(-4i64, List::<i64>::nil()))))), List::cons(List::cons(-74i64, List::cons(-40i64, List::cons(-6i64, List::cons(-31i64, List::cons(-79i64, List::<i64>::nil()))))), List::cons(List::cons(0i64, List::cons(-35i64, List::cons(-103i64, List::cons(-91i64, List::cons(-106i64, List::<i64>::nil()))))), List::cons(List::cons(-71i64, List::cons(-89i64, List::cons(-93i64, List::cons(-38i64, List::cons(-55i64, List::<i64>::nil()))))), List::cons(List::cons(-48i64, List::cons(-125i64, List::cons(-127i64, List::cons(-10i64, List::cons(-16i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(48i64, List::cons(24i64, List::cons(42i64, List::cons(66i64, List::cons(74i64, List::<i64>::nil()))))), List::cons(List::cons(-56i64, List::cons(-50i64, List::cons(-17i64, List::cons(23i64, List::cons(18i64, List::<i64>::nil()))))), List::cons(List::cons(-65i64, List::cons(-22i64, List::cons(-82i64, List::cons(-40i64, List::cons(-97i64, List::<i64>::nil()))))), List::cons(List::cons(-30i64, List::cons(-64i64, List::cons(-99i64, List::cons(-62i64, List::cons(-21i64, List::<i64>::nil()))))), List::cons(List::cons(-92i64, List::cons(-124i64, List::cons(-119i64, List::cons(-48i64, List::cons(-54i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-16i64, List::cons(-65i64, List::cons(-7i64, List::cons(33i64, List::cons(7i64, List::<i64>::nil()))))), List::cons(List::cons(-76i64, List::cons(8i64, List::cons(-12i64, List::cons(-50i64, List::cons(-25i64, List::<i64>::nil()))))), List::cons(List::cons(-63i64, List::cons(46i64, List::cons(34i64, List::cons(-1i64, List::cons(-50i64, List::<i64>::nil()))))), List::cons(List::cons(-35i64, List::cons(54i64, List::cons(28i64, List::cons(-23i64, List::cons(-11i64, List::<i64>::nil()))))), List::cons(List::cons(-72i64, List::cons(22i64, List::cons(-55i64, List::cons(50i64, List::cons(13i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-102i64, List::cons(28i64, List::cons(12i64, List::cons(7i64, List::cons(-44i64, List::<i64>::nil()))))), List::cons(List::cons(-27i64, List::cons(60i64, List::cons(64i64, List::cons(54i64, List::cons(-64i64, List::<i64>::nil()))))), List::cons(List::cons(26i64, List::cons(51i64, List::cons(-4i64, List::cons(-77i64, List::cons(-18i64, List::<i64>::nil()))))), List::cons(List::cons(68i64, List::cons(0i64, List::cons(-87i64, List::cons(-52i64, List::cons(-12i64, List::<i64>::nil()))))), List::cons(List::cons(38i64, List::cons(-113i64, List::cons(-54i64, List::cons(1i64, List::cons(53i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil())))))))), List::cons(List::cons(List::cons(List::cons(-53i64, List::cons(-15i64, List::cons(-51i64, List::cons(10i64, List::cons(22i64, List::<i64>::nil()))))), List::cons(List::cons(-40i64, List::cons(-2i64, List::cons(-72i64, List::cons(-56i64, List::cons(-45i64, List::<i64>::nil()))))), List::cons(List::cons(-15i64, List::cons(16i64, List::cons(-74i64, List::cons(-40i64, List::cons(-53i64, List::<i64>::nil()))))), List::cons(List::cons(-22i64, List::cons(-23i64, List::cons(49i64, List::cons(127i64, List::cons(25i64, List::<i64>::nil()))))), List::cons(List::cons(-2i64, List::cons(53i64, List::cons(92i64, List::cons(15i64, List::cons(11i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-57i64, List::cons(36i64, List::cons(15i64, List::cons(44i64, List::cons(21i64, List::<i64>::nil()))))), List::cons(List::cons(-4i64, List::cons(-25i64, List::cons(35i64, List::cons(69i64, List::cons(26i64, List::<i64>::nil()))))), List::cons(List::cons(-6i64, List::cons(-5i64, List::cons(-16i64, List::cons(56i64, List::cons(36i64, List::<i64>::nil()))))), List::cons(List::cons(-45i64, List::cons(-4i64, List::cons(25i64, List::cons(-64i64, List::cons(-19i64, List::<i64>::nil()))))), List::cons(List::cons(25i64, List::cons(-11i64, List::cons(-62i64, List::cons(-43i64, List::cons(26i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-39i64, List::cons(11i64, List::cons(24i64, List::cons(23i64, List::cons(-3i64, List::<i64>::nil()))))), List::cons(List::cons(-26i64, List::cons(-15i64, List::cons(5i64, List::cons(-23i64, List::cons(-89i64, List::<i64>::nil()))))), List::cons(List::cons(29i64, List::cons(38i64, List::cons(44i64, List::cons(-49i64, List::cons(-48i64, List::<i64>::nil()))))), List::cons(List::cons(48i64, List::cons(26i64, List::cons(-35i64, List::cons(-64i64, List::cons(-44i64, List::<i64>::nil()))))), List::cons(List::cons(-18i64, List::cons(-25i64, List::cons(-72i64, List::cons(-58i64, List::cons(-37i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-51i64, List::cons(2i64, List::cons(11i64, List::cons(-2i64, List::cons(-35i64, List::<i64>::nil()))))), List::cons(List::cons(-42i64, List::cons(-12i64, List::cons(7i64, List::cons(-2i64, List::cons(-42i64, List::<i64>::nil()))))), List::cons(List::cons(-21i64, List::cons(-24i64, List::cons(-5i64, List::cons(-15i64, List::cons(-1i64, List::<i64>::nil()))))), List::cons(List::cons(-45i64, List::cons(2i64, List::cons(12i64, List::cons(45i64, List::cons(90i64, List::<i64>::nil()))))), List::cons(List::cons(31i64, List::cons(55i64, List::cons(65i64, List::cons(46i64, List::cons(-18i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-5i64, List::cons(25i64, List::cons(-2i64, List::cons(-6i64, List::cons(68i64, List::<i64>::nil()))))), List::cons(List::cons(-9i64, List::cons(54i64, List::cons(43i64, List::cons(-38i64, List::cons(16i64, List::<i64>::nil()))))), List::cons(List::cons(-25i64, List::cons(-44i64, List::cons(-17i64, List::cons(-10i64, List::cons(20i64, List::<i64>::nil()))))), List::cons(List::cons(-77i64, List::cons(23i64, List::cons(-31i64, List::cons(10i64, List::cons(-34i64, List::<i64>::nil()))))), List::cons(List::cons(-13i64, List::cons(-16i64, List::cons(-10i64, List::cons(-3i64, List::cons(-26i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-5i64, List::cons(0i64, List::cons(-6i64, List::cons(-18i64, List::cons(75i64, List::<i64>::nil()))))), List::cons(List::cons(-6i64, List::cons(41i64, List::cons(40i64, List::cons(13i64, List::cons(56i64, List::<i64>::nil()))))), List::cons(List::cons(12i64, List::cons(-19i64, List::cons(-1i64, List::cons(-25i64, List::cons(-22i64, List::<i64>::nil()))))), List::cons(List::cons(-31i64, List::cons(6i64, List::cons(22i64, List::cons(-21i64, List::cons(-52i64, List::<i64>::nil()))))), List::cons(List::cons(-28i64, List::cons(20i64, List::cons(-34i64, List::cons(-71i64, List::cons(-14i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(49i64, List::cons(-2i64, List::cons(37i64, List::cons(2i64, List::cons(17i64, List::<i64>::nil()))))), List::cons(List::cons(-26i64, List::cons(-12i64, List::cons(-4i64, List::cons(27i64, List::cons(11i64, List::<i64>::nil()))))), List::cons(List::cons(-21i64, List::cons(-59i64, List::cons(-75i64, List::cons(-51i64, List::cons(-4i64, List::<i64>::nil()))))), List::cons(List::cons(-100i64, List::cons(-79i64, List::cons(-113i64, List::cons(-105i64, List::cons(-69i64, List::<i64>::nil()))))), List::cons(List::cons(-82i64, List::cons(-88i64, List::cons(-77i64, List::cons(-24i64, List::cons(28i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-47i64, List::cons(-22i64, List::cons(-3i64, List::cons(-19i64, List::cons(-8i64, List::<i64>::nil()))))), List::cons(List::cons(-23i64, List::cons(-13i64, List::cons(21i64, List::cons(-34i64, List::cons(-25i64, List::<i64>::nil()))))), List::cons(List::cons(36i64, List::cons(39i64, List::cons(-25i64, List::cons(-21i64, List::cons(-47i64, List::<i64>::nil()))))), List::cons(List::cons(-6i64, List::cons(-16i64, List::cons(12i64, List::cons(25i64, List::cons(37i64, List::<i64>::nil()))))), List::cons(List::cons(-45i64, List::cons(-25i64, List::cons(35i64, List::cons(24i64, List::cons(6i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil())))))))), List::cons(List::cons(List::cons(List::cons(-67i64, List::cons(-6i64, List::cons(-78i64, List::cons(-42i64, List::cons(46i64, List::<i64>::nil()))))), List::cons(List::cons(25i64, List::cons(15i64, List::cons(-60i64, List::cons(12i64, List::cons(61i64, List::<i64>::nil()))))), List::cons(List::cons(40i64, List::cons(1i64, List::cons(-30i64, List::cons(36i64, List::cons(3i64, List::<i64>::nil()))))), List::cons(List::cons(68i64, List::cons(-94i64, List::cons(-25i64, List::cons(-41i64, List::cons(-61i64, List::<i64>::nil()))))), List::cons(List::cons(5i64, List::cons(-28i64, List::cons(-103i64, List::cons(-118i64, List::cons(-33i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-6i64, List::cons(33i64, List::cons(49i64, List::cons(11i64, List::cons(-51i64, List::<i64>::nil()))))), List::cons(List::cons(2i64, List::cons(-18i64, List::cons(33i64, List::cons(116i64, List::cons(-31i64, List::<i64>::nil()))))), List::cons(List::cons(29i64, List::cons(-45i64, List::cons(40i64, List::cons(84i64, List::cons(6i64, List::<i64>::nil()))))), List::cons(List::cons(-43i64, List::cons(-1i64, List::cons(37i64, List::cons(127i64, List::cons(20i64, List::<i64>::nil()))))), List::cons(List::cons(-37i64, List::cons(-38i64, List::cons(1i64, List::cons(58i64, List::cons(3i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(29i64, List::cons(-5i64, List::cons(-29i64, List::cons(-19i64, List::cons(-24i64, List::<i64>::nil()))))), List::cons(List::cons(34i64, List::cons(54i64, List::cons(-7i64, List::cons(-5i64, List::cons(-50i64, List::<i64>::nil()))))), List::cons(List::cons(-24i64, List::cons(-8i64, List::cons(-55i64, List::cons(-35i64, List::cons(-102i64, List::<i64>::nil()))))), List::cons(List::cons(-32i64, List::cons(-29i64, List::cons(-26i64, List::cons(42i64, List::cons(-43i64, List::<i64>::nil()))))), List::cons(List::cons(-64i64, List::cons(-41i64, List::cons(-42i64, List::cons(28i64, List::cons(5i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(66i64, List::cons(30i64, List::cons(-10i64, List::cons(-15i64, List::cons(-19i64, List::<i64>::nil()))))), List::cons(List::cons(38i64, List::cons(46i64, List::cons(-55i64, List::cons(33i64, List::cons(-25i64, List::<i64>::nil()))))), List::cons(List::cons(13i64, List::cons(-44i64, List::cons(-34i64, List::cons(6i64, List::cons(-46i64, List::<i64>::nil()))))), List::cons(List::cons(9i64, List::cons(-81i64, List::cons(-72i64, List::cons(6i64, List::cons(-32i64, List::<i64>::nil()))))), List::cons(List::cons(-15i64, List::cons(-59i64, List::cons(-14i64, List::cons(-19i64, List::cons(-74i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(11i64, List::cons(-50i64, List::cons(-27i64, List::cons(-27i64, List::cons(-18i64, List::<i64>::nil()))))), List::cons(List::cons(14i64, List::cons(23i64, List::cons(-40i64, List::cons(-42i64, List::cons(30i64, List::<i64>::nil()))))), List::cons(List::cons(-33i64, List::cons(-27i64, List::cons(-76i64, List::cons(-8i64, List::cons(48i64, List::<i64>::nil()))))), List::cons(List::cons(-25i64, List::cons(-33i64, List::cons(-41i64, List::cons(-4i64, List::cons(105i64, List::<i64>::nil()))))), List::cons(List::cons(-30i64, List::cons(-21i64, List::cons(-4i64, List::cons(7i64, List::cons(39i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(25i64, List::cons(-1i64, List::cons(-87i64, List::cons(-57i64, List::cons(-36i64, List::<i64>::nil()))))), List::cons(List::cons(32i64, List::cons(8i64, List::cons(-15i64, List::cons(-78i64, List::cons(-14i64, List::<i64>::nil()))))), List::cons(List::cons(-25i64, List::cons(-30i64, List::cons(-21i64, List::cons(-59i64, List::cons(-42i64, List::<i64>::nil()))))), List::cons(List::cons(3i64, List::cons(-37i64, List::cons(-40i64, List::cons(-40i64, List::cons(33i64, List::<i64>::nil()))))), List::cons(List::cons(-9i64, List::cons(-51i64, List::cons(-29i64, List::cons(18i64, List::cons(52i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-57i64, List::cons(-63i64, List::cons(-99i64, List::cons(-50i64, List::cons(69i64, List::<i64>::nil()))))), List::cons(List::cons(-27i64, List::cons(-4i64, List::cons(-15i64, List::cons(-18i64, List::cons(82i64, List::<i64>::nil()))))), List::cons(List::cons(-11i64, List::cons(45i64, List::cons(3i64, List::cons(20i64, List::cons(90i64, List::<i64>::nil()))))), List::cons(List::cons(29i64, List::cons(82i64, List::cons(41i64, List::cons(45i64, List::cons(103i64, List::<i64>::nil()))))), List::cons(List::cons(53i64, List::cons(20i64, List::cons(27i64, List::cons(-24i64, List::cons(5i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(27i64, List::cons(-16i64, List::cons(26i64, List::cons(9i64, List::cons(12i64, List::<i64>::nil()))))), List::cons(List::cons(-13i64, List::cons(61i64, List::cons(-4i64, List::cons(-41i64, List::cons(5i64, List::<i64>::nil()))))), List::cons(List::cons(34i64, List::cons(12i64, List::cons(-1i64, List::cons(-1i64, List::cons(-53i64, List::<i64>::nil()))))), List::cons(List::cons(22i64, List::cons(-61i64, List::cons(-35i64, List::cons(49i64, List::cons(-51i64, List::<i64>::nil()))))), List::cons(List::cons(-9i64, List::cons(-91i64, List::cons(-60i64, List::cons(-47i64, List::cons(1i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil())))))))), List::cons(List::cons(List::cons(List::cons(20i64, List::cons(25i64, List::cons(-12i64, List::cons(-5i64, List::cons(-35i64, List::<i64>::nil()))))), List::cons(List::cons(35i64, List::cons(49i64, List::cons(-52i64, List::cons(-31i64, List::cons(-23i64, List::<i64>::nil()))))), List::cons(List::cons(-16i64, List::cons(-41i64, List::cons(8i64, List::cons(9i64, List::cons(-10i64, List::<i64>::nil()))))), List::cons(List::cons(-68i64, List::cons(9i64, List::cons(44i64, List::cons(38i64, List::cons(-16i64, List::<i64>::nil()))))), List::cons(List::cons(12i64, List::cons(87i64, List::cons(33i64, List::cons(50i64, List::cons(108i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(6i64, List::cons(-22i64, List::cons(-29i64, List::cons(-45i64, List::cons(-16i64, List::<i64>::nil()))))), List::cons(List::cons(22i64, List::cons(-49i64, List::cons(-2i64, List::cons(5i64, List::cons(-14i64, List::<i64>::nil()))))), List::cons(List::cons(34i64, List::cons(-75i64, List::cons(-32i64, List::cons(-24i64, List::cons(19i64, List::<i64>::nil()))))), List::cons(List::cons(58i64, List::cons(-7i64, List::cons(17i64, List::cons(25i64, List::cons(-25i64, List::<i64>::nil()))))), List::cons(List::cons(40i64, List::cons(15i64, List::cons(36i64, List::cons(53i64, List::cons(-1i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-28i64, List::cons(-56i64, List::cons(-59i64, List::cons(-65i64, List::cons(-9i64, List::<i64>::nil()))))), List::cons(List::cons(-34i64, List::cons(-51i64, List::cons(-25i64, List::cons(-17i64, List::cons(6i64, List::<i64>::nil()))))), List::cons(List::cons(-15i64, List::cons(-8i64, List::cons(-20i64, List::cons(7i64, List::cons(21i64, List::<i64>::nil()))))), List::cons(List::cons(-42i64, List::cons(25i64, List::cons(54i64, List::cons(12i64, List::cons(-75i64, List::<i64>::nil()))))), List::cons(List::cons(46i64, List::cons(74i64, List::cons(103i64, List::cons(-61i64, List::cons(-127i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(0i64, List::cons(-16i64, List::cons(-37i64, List::cons(-39i64, List::cons(40i64, List::<i64>::nil()))))), List::cons(List::cons(2i64, List::cons(-13i64, List::cons(-14i64, List::cons(23i64, List::cons(-4i64, List::<i64>::nil()))))), List::cons(List::cons(19i64, List::cons(-8i64, List::cons(28i64, List::cons(9i64, List::cons(30i64, List::<i64>::nil()))))), List::cons(List::cons(-26i64, List::cons(8i64, List::cons(-17i64, List::cons(-23i64, List::cons(-19i64, List::<i64>::nil()))))), List::cons(List::cons(-78i64, List::cons(-45i64, List::cons(-82i64, List::cons(-34i64, List::cons(29i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-2i64, List::cons(-61i64, List::cons(-72i64, List::cons(-30i64, List::cons(-1i64, List::<i64>::nil()))))), List::cons(List::cons(-65i64, List::cons(-42i64, List::cons(-31i64, List::cons(1i64, List::cons(3i64, List::<i64>::nil()))))), List::cons(List::cons(16i64, List::cons(84i64, List::cons(10i64, List::cons(-22i64, List::cons(-19i64, List::<i64>::nil()))))), List::cons(List::cons(62i64, List::cons(4i64, List::cons(-44i64, List::cons(-65i64, List::cons(-36i64, List::<i64>::nil()))))), List::cons(List::cons(-52i64, List::cons(-16i64, List::cons(-80i64, List::cons(-53i64, List::cons(19i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-16i64, List::cons(-6i64, List::cons(-55i64, List::cons(2i64, List::cons(-25i64, List::<i64>::nil()))))), List::cons(List::cons(-55i64, List::cons(-29i64, List::cons(-54i64, List::cons(-36i64, List::cons(-12i64, List::<i64>::nil()))))), List::cons(List::cons(55i64, List::cons(69i64, List::cons(19i64, List::cons(-11i64, List::cons(-33i64, List::<i64>::nil()))))), List::cons(List::cons(35i64, List::cons(15i64, List::cons(-16i64, List::cons(-62i64, List::cons(-38i64, List::<i64>::nil()))))), List::cons(List::cons(-62i64, List::cons(-42i64, List::cons(-52i64, List::cons(-89i64, List::cons(-37i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-1i64, List::cons(-11i64, List::cons(-3i64, List::cons(-52i64, List::cons(-11i64, List::<i64>::nil()))))), List::cons(List::cons(4i64, List::cons(10i64, List::cons(37i64, List::cons(-24i64, List::cons(-11i64, List::<i64>::nil()))))), List::cons(List::cons(-9i64, List::cons(44i64, List::cons(-1i64, List::cons(-39i64, List::cons(1i64, List::<i64>::nil()))))), List::cons(List::cons(16i64, List::cons(-11i64, List::cons(-21i64, List::cons(-32i64, List::cons(-26i64, List::<i64>::nil()))))), List::cons(List::cons(-32i64, List::cons(-7i64, List::cons(11i64, List::cons(-25i64, List::cons(21i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-30i64, List::cons(-76i64, List::cons(-57i64, List::cons(-39i64, List::cons(38i64, List::<i64>::nil()))))), List::cons(List::cons(0i64, List::cons(-55i64, List::cons(-46i64, List::cons(-29i64, List::cons(-30i64, List::<i64>::nil()))))), List::cons(List::cons(-29i64, List::cons(-24i64, List::cons(-26i64, List::cons(-8i64, List::cons(-9i64, List::<i64>::nil()))))), List::cons(List::cons(6i64, List::cons(62i64, List::cons(66i64, List::cons(0i64, List::cons(-38i64, List::<i64>::nil()))))), List::cons(List::cons(76i64, List::cons(72i64, List::cons(28i64, List::cons(-82i64, List::cons(-26i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil())))))))), List::cons(List::cons(List::cons(List::cons(-65i64, List::cons(38i64, List::cons(62i64, List::cons(-48i64, List::cons(-17i64, List::<i64>::nil()))))), List::cons(List::cons(-46i64, List::cons(-13i64, List::cons(-34i64, List::cons(-21i64, List::cons(-18i64, List::<i64>::nil()))))), List::cons(List::cons(-33i64, List::cons(-16i64, List::cons(20i64, List::cons(-23i64, List::cons(-43i64, List::<i64>::nil()))))), List::cons(List::cons(26i64, List::cons(7i64, List::cons(-8i64, List::cons(30i64, List::cons(6i64, List::<i64>::nil()))))), List::cons(List::cons(64i64, List::cons(37i64, List::cons(-46i64, List::cons(69i64, List::cons(41i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-42i64, List::cons(-1i64, List::cons(-28i64, List::cons(9i64, List::cons(-35i64, List::<i64>::nil()))))), List::cons(List::cons(4i64, List::cons(-27i64, List::cons(11i64, List::cons(23i64, List::cons(-68i64, List::<i64>::nil()))))), List::cons(List::cons(5i64, List::cons(-1i64, List::cons(-6i64, List::cons(-10i64, List::cons(24i64, List::<i64>::nil()))))), List::cons(List::cons(-32i64, List::cons(-18i64, List::cons(-21i64, List::cons(2i64, List::cons(34i64, List::<i64>::nil()))))), List::cons(List::cons(-9i64, List::cons(-75i64, List::cons(24i64, List::cons(25i64, List::cons(14i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(55i64, List::cons(77i64, List::cons(11i64, List::cons(-46i64, List::cons(6i64, List::<i64>::nil()))))), List::cons(List::cons(-9i64, List::cons(21i64, List::cons(8i64, List::cons(-32i64, List::cons(-35i64, List::<i64>::nil()))))), List::cons(List::cons(9i64, List::cons(17i64, List::cons(34i64, List::cons(-28i64, List::cons(-64i64, List::<i64>::nil()))))), List::cons(List::cons(2i64, List::cons(16i64, List::cons(53i64, List::cons(4i64, List::cons(-14i64, List::<i64>::nil()))))), List::cons(List::cons(2i64, List::cons(6i64, List::cons(-41i64, List::cons(-61i64, List::cons(-47i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-23i64, List::cons(-22i64, List::cons(7i64, List::cons(-54i64, List::cons(-84i64, List::<i64>::nil()))))), List::cons(List::cons(-71i64, List::cons(-56i64, List::cons(-34i64, List::cons(-6i64, List::cons(-42i64, List::<i64>::nil()))))), List::cons(List::cons(-40i64, List::cons(-28i64, List::cons(27i64, List::cons(11i64, List::cons(45i64, List::<i64>::nil()))))), List::cons(List::cons(-15i64, List::cons(-11i64, List::cons(30i64, List::cons(16i64, List::cons(107i64, List::<i64>::nil()))))), List::cons(List::cons(10i64, List::cons(7i64, List::cons(35i64, List::cons(111i64, List::cons(127i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-2i64, List::cons(7i64, List::cons(7i64, List::cons(-7i64, List::cons(-34i64, List::<i64>::nil()))))), List::cons(List::cons(2i64, List::cons(26i64, List::cons(16i64, List::cons(-24i64, List::cons(8i64, List::<i64>::nil()))))), List::cons(List::cons(-29i64, List::cons(22i64, List::cons(4i64, List::cons(10i64, List::cons(66i64, List::<i64>::nil()))))), List::cons(List::cons(0i64, List::cons(-23i64, List::cons(-14i64, List::cons(-17i64, List::cons(25i64, List::<i64>::nil()))))), List::cons(List::cons(3i64, List::cons(11i64, List::cons(-43i64, List::cons(-3i64, List::cons(-14i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(47i64, List::cons(14i64, List::cons(-27i64, List::cons(-3i64, List::cons(13i64, List::<i64>::nil()))))), List::cons(List::cons(-42i64, List::cons(15i64, List::cons(27i64, List::cons(-36i64, List::cons(7i64, List::<i64>::nil()))))), List::cons(List::cons(6i64, List::cons(20i64, List::cons(33i64, List::cons(-11i64, List::cons(33i64, List::<i64>::nil()))))), List::cons(List::cons(5i64, List::cons(-25i64, List::cons(12i64, List::cons(10i64, List::cons(-29i64, List::<i64>::nil()))))), List::cons(List::cons(-4i64, List::cons(8i64, List::cons(0i64, List::cons(-37i64, List::cons(-100i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(38i64, List::cons(-19i64, List::cons(52i64, List::cons(57i64, List::cons(-2i64, List::<i64>::nil()))))), List::cons(List::cons(24i64, List::cons(9i64, List::cons(5i64, List::cons(4i64, List::cons(11i64, List::<i64>::nil()))))), List::cons(List::cons(-16i64, List::cons(-4i64, List::cons(-27i64, List::cons(-36i64, List::cons(-14i64, List::<i64>::nil()))))), List::cons(List::cons(14i64, List::cons(-37i64, List::cons(-33i64, List::cons(-34i64, List::cons(-34i64, List::<i64>::nil()))))), List::cons(List::cons(29i64, List::cons(24i64, List::cons(-16i64, List::cons(-20i64, List::cons(-28i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(57i64, List::cons(57i64, List::cons(19i64, List::cons(-69i64, List::cons(-43i64, List::<i64>::nil()))))), List::cons(List::cons(5i64, List::cons(12i64, List::cons(29i64, List::cons(-44i64, List::cons(-67i64, List::<i64>::nil()))))), List::cons(List::cons(0i64, List::cons(-25i64, List::cons(-19i64, List::cons(-27i64, List::cons(-57i64, List::<i64>::nil()))))), List::cons(List::cons(16i64, List::cons(4i64, List::cons(0i64, List::cons(-36i64, List::cons(-58i64, List::<i64>::nil()))))), List::cons(List::cons(7i64, List::cons(-11i64, List::cons(-57i64, List::cons(-33i64, List::cons(7i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil())))))))), List::cons(List::cons(List::cons(List::cons(17i64, List::cons(52i64, List::cons(-53i64, List::cons(-75i64, List::cons(-43i64, List::<i64>::nil()))))), List::cons(List::cons(90i64, List::cons(84i64, List::cons(-18i64, List::cons(21i64, List::cons(-24i64, List::<i64>::nil()))))), List::cons(List::cons(-33i64, List::cons(-35i64, List::cons(-48i64, List::cons(-58i64, List::cons(-8i64, List::<i64>::nil()))))), List::cons(List::cons(-92i64, List::cons(-56i64, List::cons(-23i64, List::cons(-59i64, List::cons(-113i64, List::<i64>::nil()))))), List::cons(List::cons(-21i64, List::cons(17i64, List::cons(-72i64, List::cons(-84i64, List::cons(-105i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(41i64, List::cons(14i64, List::cons(76i64, List::cons(67i64, List::cons(30i64, List::<i64>::nil()))))), List::cons(List::cons(8i64, List::cons(-110i64, List::cons(31i64, List::cons(37i64, List::cons(65i64, List::<i64>::nil()))))), List::cons(List::cons(66i64, List::cons(-76i64, List::cons(-42i64, List::cons(1i64, List::cons(21i64, List::<i64>::nil()))))), List::cons(List::cons(16i64, List::cons(-39i64, List::cons(14i64, List::cons(-77i64, List::cons(-55i64, List::<i64>::nil()))))), List::cons(List::cons(53i64, List::cons(-4i64, List::cons(3i64, List::cons(-91i64, List::cons(31i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-2i64, List::cons(-37i64, List::cons(21i64, List::cons(6i64, List::cons(18i64, List::<i64>::nil()))))), List::cons(List::cons(-34i64, List::cons(20i64, List::cons(34i64, List::cons(-2i64, List::cons(5i64, List::<i64>::nil()))))), List::cons(List::cons(102i64, List::cons(16i64, List::cons(-63i64, List::cons(-3i64, List::cons(49i64, List::<i64>::nil()))))), List::cons(List::cons(71i64, List::cons(-39i64, List::cons(-88i64, List::cons(-22i64, List::cons(-1i64, List::<i64>::nil()))))), List::cons(List::cons(-17i64, List::cons(-17i64, List::cons(8i64, List::cons(1i64, List::cons(-83i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(115i64, List::cons(3i64, List::cons(-105i64, List::cons(-91i64, List::cons(22i64, List::<i64>::nil()))))), List::cons(List::cons(50i64, List::cons(-66i64, List::cons(-127i64, List::cons(-82i64, List::cons(-12i64, List::<i64>::nil()))))), List::cons(List::cons(-99i64, List::cons(-61i64, List::cons(-91i64, List::cons(-62i64, List::cons(-35i64, List::<i64>::nil()))))), List::cons(List::cons(-84i64, List::cons(-60i64, List::cons(-64i64, List::cons(-25i64, List::cons(-65i64, List::<i64>::nil()))))), List::cons(List::cons(21i64, List::cons(26i64, List::cons(-9i64, List::cons(7i64, List::cons(82i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(24i64, List::cons(7i64, List::cons(-48i64, List::cons(1i64, List::cons(-74i64, List::<i64>::nil()))))), List::cons(List::cons(-12i64, List::cons(-32i64, List::cons(-65i64, List::cons(-88i64, List::cons(4i64, List::<i64>::nil()))))), List::cons(List::cons(-14i64, List::cons(-37i64, List::cons(-25i64, List::cons(13i64, List::cons(-65i64, List::<i64>::nil()))))), List::cons(List::cons(-7i64, List::cons(19i64, List::cons(-44i64, List::cons(30i64, List::cons(-61i64, List::<i64>::nil()))))), List::cons(List::cons(-35i64, List::cons(27i64, List::cons(13i64, List::cons(36i64, List::cons(97i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(3i64, List::cons(-78i64, List::cons(-4i64, List::cons(-6i64, List::cons(-26i64, List::<i64>::nil()))))), List::cons(List::cons(-97i64, List::cons(-20i64, List::cons(-15i64, List::cons(-78i64, List::cons(2i64, List::<i64>::nil()))))), List::cons(List::cons(-28i64, List::cons(42i64, List::cons(-26i64, List::cons(-19i64, List::cons(-14i64, List::<i64>::nil()))))), List::cons(List::cons(-8i64, List::cons(6i64, List::cons(21i64, List::cons(-20i64, List::cons(-59i64, List::<i64>::nil()))))), List::cons(List::cons(-19i64, List::cons(-15i64, List::cons(-18i64, List::cons(13i64, List::cons(107i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-89i64, List::cons(-2i64, List::cons(68i64, List::cons(-79i64, List::cons(-1i64, List::<i64>::nil()))))), List::cons(List::cons(-7i64, List::cons(86i64, List::cons(15i64, List::cons(-29i64, List::cons(23i64, List::<i64>::nil()))))), List::cons(List::cons(111i64, List::cons(69i64, List::cons(-49i64, List::cons(-31i64, List::cons(-57i64, List::<i64>::nil()))))), List::cons(List::cons(-30i64, List::cons(-98i64, List::cons(-33i64, List::cons(14i64, List::cons(-90i64, List::<i64>::nil()))))), List::cons(List::cons(-98i64, List::cons(-51i64, List::cons(-48i64, List::cons(79i64, List::cons(86i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(33i64, List::cons(100i64, List::cons(-29i64, List::cons(11i64, List::cons(36i64, List::<i64>::nil()))))), List::cons(List::cons(114i64, List::cons(60i64, List::cons(-4i64, List::cons(5i64, List::cons(45i64, List::<i64>::nil()))))), List::cons(List::cons(104i64, List::cons(-43i64, List::cons(-117i64, List::cons(-32i64, List::cons(27i64, List::<i64>::nil()))))), List::cons(List::cons(-42i64, List::cons(-83i64, List::cons(-51i64, List::cons(-67i64, List::cons(-34i64, List::<i64>::nil()))))), List::cons(List::cons(-40i64, List::cons(8i64, List::cons(-11i64, List::cons(-45i64, List::cons(-93i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil())))))))), List::cons(List::cons(List::cons(List::cons(21i64, List::cons(-9i64, List::cons(-47i64, List::cons(-44i64, List::cons(-40i64, List::<i64>::nil()))))), List::cons(List::cons(-26i64, List::cons(36i64, List::cons(-40i64, List::cons(-50i64, List::cons(13i64, List::<i64>::nil()))))), List::cons(List::cons(-16i64, List::cons(1i64, List::cons(46i64, List::cons(127i64, List::cons(45i64, List::<i64>::nil()))))), List::cons(List::cons(-30i64, List::cons(65i64, List::cons(61i64, List::cons(71i64, List::cons(26i64, List::<i64>::nil()))))), List::cons(List::cons(23i64, List::cons(18i64, List::cons(59i64, List::cons(36i64, List::cons(-11i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-21i64, List::cons(-13i64, List::cons(-28i64, List::cons(-59i64, List::cons(-32i64, List::<i64>::nil()))))), List::cons(List::cons(7i64, List::cons(-14i64, List::cons(2i64, List::cons(53i64, List::cons(-45i64, List::<i64>::nil()))))), List::cons(List::cons(15i64, List::cons(7i64, List::cons(73i64, List::cons(34i64, List::cons(-4i64, List::<i64>::nil()))))), List::cons(List::cons(45i64, List::cons(-40i64, List::cons(35i64, List::cons(-26i64, List::cons(-38i64, List::<i64>::nil()))))), List::cons(List::cons(1i64, List::cons(-26i64, List::cons(7i64, List::cons(-30i64, List::cons(-47i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(2i64, List::cons(-4i64, List::cons(-8i64, List::cons(-33i64, List::cons(-50i64, List::<i64>::nil()))))), List::cons(List::cons(27i64, List::cons(21i64, List::cons(9i64, List::cons(-16i64, List::cons(-68i64, List::<i64>::nil()))))), List::cons(List::cons(21i64, List::cons(29i64, List::cons(-27i64, List::cons(-46i64, List::cons(-23i64, List::<i64>::nil()))))), List::cons(List::cons(19i64, List::cons(-66i64, List::cons(-73i64, List::cons(-31i64, List::cons(17i64, List::<i64>::nil()))))), List::cons(List::cons(13i64, List::cons(11i64, List::cons(-38i64, List::cons(-38i64, List::cons(-58i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(5i64, List::cons(20i64, List::cons(22i64, List::cons(20i64, List::cons(-29i64, List::<i64>::nil()))))), List::cons(List::cons(1i64, List::cons(-8i64, List::cons(4i64, List::cons(-38i64, List::cons(-29i64, List::<i64>::nil()))))), List::cons(List::cons(1i64, List::cons(-29i64, List::cons(-60i64, List::cons(-51i64, List::cons(-11i64, List::<i64>::nil()))))), List::cons(List::cons(11i64, List::cons(-11i64, List::cons(30i64, List::cons(18i64, List::cons(-16i64, List::<i64>::nil()))))), List::cons(List::cons(21i64, List::cons(12i64, List::cons(23i64, List::cons(2i64, List::cons(-26i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(3i64, List::cons(30i64, List::cons(26i64, List::cons(-41i64, List::cons(-17i64, List::<i64>::nil()))))), List::cons(List::cons(33i64, List::cons(11i64, List::cons(-20i64, List::cons(-36i64, List::cons(-8i64, List::<i64>::nil()))))), List::cons(List::cons(2i64, List::cons(-46i64, List::cons(-19i64, List::cons(-5i64, List::cons(-21i64, List::<i64>::nil()))))), List::cons(List::cons(-16i64, List::cons(-31i64, List::cons(-24i64, List::cons(26i64, List::cons(19i64, List::<i64>::nil()))))), List::cons(List::cons(3i64, List::cons(-30i64, List::cons(-9i64, List::cons(30i64, List::cons(6i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(2i64, List::cons(27i64, List::cons(27i64, List::cons(-15i64, List::cons(11i64, List::<i64>::nil()))))), List::cons(List::cons(26i64, List::cons(17i64, List::cons(11i64, List::cons(-19i64, List::cons(-26i64, List::<i64>::nil()))))), List::cons(List::cons(-3i64, List::cons(-27i64, List::cons(-20i64, List::cons(37i64, List::cons(-28i64, List::<i64>::nil()))))), List::cons(List::cons(35i64, List::cons(-23i64, List::cons(-64i64, List::cons(-44i64, List::cons(33i64, List::<i64>::nil()))))), List::cons(List::cons(-26i64, List::cons(-36i64, List::cons(-53i64, List::cons(-32i64, List::cons(19i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(1i64, List::cons(7i64, List::cons(-47i64, List::cons(-44i64, List::cons(-8i64, List::<i64>::nil()))))), List::cons(List::cons(-35i64, List::cons(-44i64, List::cons(-72i64, List::cons(-53i64, List::cons(7i64, List::<i64>::nil()))))), List::cons(List::cons(-70i64, List::cons(-75i64, List::cons(-80i64, List::cons(16i64, List::cons(46i64, List::<i64>::nil()))))), List::cons(List::cons(-38i64, List::cons(-56i64, List::cons(-49i64, List::cons(65i64, List::cons(90i64, List::<i64>::nil()))))), List::cons(List::cons(-27i64, List::cons(10i64, List::cons(74i64, List::cons(81i64, List::cons(-57i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(24i64, List::cons(-6i64, List::cons(-44i64, List::cons(-45i64, List::cons(-57i64, List::<i64>::nil()))))), List::cons(List::cons(22i64, List::cons(52i64, List::cons(16i64, List::cons(28i64, List::cons(27i64, List::<i64>::nil()))))), List::cons(List::cons(35i64, List::cons(7i64, List::cons(-20i64, List::cons(38i64, List::cons(36i64, List::<i64>::nil()))))), List::cons(List::cons(-28i64, List::cons(-47i64, List::cons(-11i64, List::cons(23i64, List::cons(-32i64, List::<i64>::nil()))))), List::cons(List::cons(33i64, List::cons(-18i64, List::cons(-21i64, List::cons(-7i64, List::cons(-20i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil())))))))), List::cons(List::cons(List::cons(List::cons(72i64, List::cons(20i64, List::cons(27i64, List::cons(-16i64, List::cons(-10i64, List::<i64>::nil()))))), List::cons(List::cons(-13i64, List::cons(-8i64, List::cons(-39i64, List::cons(14i64, List::cons(-11i64, List::<i64>::nil()))))), List::cons(List::cons(-81i64, List::cons(-12i64, List::cons(-33i64, List::cons(23i64, List::cons(-6i64, List::<i64>::nil()))))), List::cons(List::cons(34i64, List::cons(-36i64, List::cons(-47i64, List::cons(-48i64, List::cons(-94i64, List::<i64>::nil()))))), List::cons(List::cons(107i64, List::cons(-14i64, List::cons(-72i64, List::cons(-14i64, List::cons(-44i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-20i64, List::cons(16i64, List::cons(30i64, List::cons(36i64, List::cons(79i64, List::<i64>::nil()))))), List::cons(List::cons(-55i64, List::cons(-78i64, List::cons(-52i64, List::cons(-26i64, List::cons(-2i64, List::<i64>::nil()))))), List::cons(List::cons(19i64, List::cons(-25i64, List::cons(-9i64, List::cons(57i64, List::cons(28i64, List::<i64>::nil()))))), List::cons(List::cons(61i64, List::cons(18i64, List::cons(10i64, List::cons(49i64, List::cons(34i64, List::<i64>::nil()))))), List::cons(List::cons(90i64, List::cons(11i64, List::cons(3i64, List::cons(94i64, List::cons(74i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-26i64, List::cons(46i64, List::cons(27i64, List::cons(71i64, List::cons(75i64, List::<i64>::nil()))))), List::cons(List::cons(-33i64, List::cons(-21i64, List::cons(-3i64, List::cons(-61i64, List::cons(-66i64, List::<i64>::nil()))))), List::cons(List::cons(-62i64, List::cons(-37i64, List::cons(6i64, List::cons(48i64, List::cons(56i64, List::<i64>::nil()))))), List::cons(List::cons(-12i64, List::cons(-35i64, List::cons(-20i64, List::cons(-15i64, List::cons(2i64, List::<i64>::nil()))))), List::cons(List::cons(-11i64, List::cons(-47i64, List::cons(-71i64, List::cons(-92i64, List::cons(-62i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-33i64, List::cons(14i64, List::cons(0i64, List::cons(-31i64, List::cons(-30i64, List::<i64>::nil()))))), List::cons(List::cons(29i64, List::cons(-8i64, List::cons(-33i64, List::cons(2i64, List::cons(15i64, List::<i64>::nil()))))), List::cons(List::cons(-48i64, List::cons(-61i64, List::cons(-64i64, List::cons(-27i64, List::cons(41i64, List::<i64>::nil()))))), List::cons(List::cons(7i64, List::cons(-26i64, List::cons(-95i64, List::cons(-33i64, List::cons(83i64, List::<i64>::nil()))))), List::cons(List::cons(62i64, List::cons(-6i64, List::cons(-85i64, List::cons(-5i64, List::cons(40i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-100i64, List::cons(-6i64, List::cons(-16i64, List::cons(-56i64, List::cons(-23i64, List::<i64>::nil()))))), List::cons(List::cons(25i64, List::cons(-16i64, List::cons(-37i64, List::cons(1i64, List::cons(29i64, List::<i64>::nil()))))), List::cons(List::cons(14i64, List::cons(64i64, List::cons(-68i64, List::cons(-50i64, List::cons(-8i64, List::<i64>::nil()))))), List::cons(List::cons(-72i64, List::cons(30i64, List::cons(19i64, List::cons(-38i64, List::cons(60i64, List::<i64>::nil()))))), List::cons(List::cons(-26i64, List::cons(16i64, List::cons(-73i64, List::cons(-34i64, List::cons(63i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-63i64, List::cons(-28i64, List::cons(-9i64, List::cons(-37i64, List::cons(-34i64, List::<i64>::nil()))))), List::cons(List::cons(18i64, List::cons(-54i64, List::cons(-22i64, List::cons(33i64, List::cons(10i64, List::<i64>::nil()))))), List::cons(List::cons(1i64, List::cons(88i64, List::cons(19i64, List::cons(-79i64, List::cons(2i64, List::<i64>::nil()))))), List::cons(List::cons(-59i64, List::cons(67i64, List::cons(23i64, List::cons(0i64, List::cons(21i64, List::<i64>::nil()))))), List::cons(List::cons(-42i64, List::cons(-49i64, List::cons(-51i64, List::cons(-62i64, List::cons(50i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-63i64, List::cons(-59i64, List::cons(1i64, List::cons(-47i64, List::cons(-18i64, List::<i64>::nil()))))), List::cons(List::cons(8i64, List::cons(-19i64, List::cons(4i64, List::cons(-42i64, List::cons(-14i64, List::<i64>::nil()))))), List::cons(List::cons(72i64, List::cons(33i64, List::cons(28i64, List::cons(-21i64, List::cons(-36i64, List::<i64>::nil()))))), List::cons(List::cons(0i64, List::cons(99i64, List::cons(32i64, List::cons(-16i64, List::cons(7i64, List::<i64>::nil()))))), List::cons(List::cons(27i64, List::cons(127i64, List::cons(18i64, List::cons(-57i64, List::cons(28i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(39i64, List::cons(52i64, List::cons(80i64, List::cons(64i64, List::cons(47i64, List::<i64>::nil()))))), List::cons(List::cons(-39i64, List::cons(-43i64, List::cons(-74i64, List::cons(-55i64, List::cons(-78i64, List::<i64>::nil()))))), List::cons(List::cons(-65i64, List::cons(-33i64, List::cons(-14i64, List::cons(-17i64, List::cons(-10i64, List::<i64>::nil()))))), List::cons(List::cons(-33i64, List::cons(-31i64, List::cons(-67i64, List::cons(-22i64, List::cons(-39i64, List::<i64>::nil()))))), List::cons(List::cons(28i64, List::cons(13i64, List::cons(-83i64, List::cons(-53i64, List::cons(-12i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil())))))))), List::cons(List::cons(List::cons(List::cons(-78i64, List::cons(-8i64, List::cons(10i64, List::cons(-43i64, List::cons(11i64, List::<i64>::nil()))))), List::cons(List::cons(-93i64, List::cons(42i64, List::cons(64i64, List::cons(-7i64, List::cons(-58i64, List::<i64>::nil()))))), List::cons(List::cons(-38i64, List::cons(51i64, List::cons(51i64, List::cons(-8i64, List::cons(-18i64, List::<i64>::nil()))))), List::cons(List::cons(44i64, List::cons(52i64, List::cons(-17i64, List::cons(-57i64, List::cons(-33i64, List::<i64>::nil()))))), List::cons(List::cons(30i64, List::cons(19i64, List::cons(7i64, List::cons(-14i64, List::cons(-20i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-47i64, List::cons(88i64, List::cons(28i64, List::cons(-15i64, List::cons(-60i64, List::<i64>::nil()))))), List::cons(List::cons(-20i64, List::cons(110i64, List::cons(21i64, List::cons(-26i64, List::cons(-26i64, List::<i64>::nil()))))), List::cons(List::cons(32i64, List::cons(91i64, List::cons(-12i64, List::cons(-51i64, List::cons(-26i64, List::<i64>::nil()))))), List::cons(List::cons(24i64, List::cons(52i64, List::cons(-5i64, List::cons(-77i64, List::cons(-12i64, List::<i64>::nil()))))), List::cons(List::cons(41i64, List::cons(-18i64, List::cons(-69i64, List::cons(-49i64, List::cons(-19i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-24i64, List::cons(4i64, List::cons(9i64, List::cons(-53i64, List::cons(5i64, List::<i64>::nil()))))), List::cons(List::cons(15i64, List::cons(-32i64, List::cons(-54i64, List::cons(-15i64, List::cons(35i64, List::<i64>::nil()))))), List::cons(List::cons(-72i64, List::cons(-52i64, List::cons(-76i64, List::cons(-37i64, List::cons(32i64, List::<i64>::nil()))))), List::cons(List::cons(-58i64, List::cons(-54i64, List::cons(-36i64, List::cons(-36i64, List::cons(-56i64, List::<i64>::nil()))))), List::cons(List::cons(-90i64, List::cons(-27i64, List::cons(-17i64, List::cons(2i64, List::cons(-35i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-97i64, List::cons(14i64, List::cons(19i64, List::cons(15i64, List::cons(-40i64, List::<i64>::nil()))))), List::cons(List::cons(-24i64, List::cons(35i64, List::cons(30i64, List::cons(-70i64, List::cons(-29i64, List::<i64>::nil()))))), List::cons(List::cons(-9i64, List::cons(89i64, List::cons(23i64, List::cons(-70i64, List::cons(-52i64, List::<i64>::nil()))))), List::cons(List::cons(-5i64, List::cons(74i64, List::cons(0i64, List::cons(-82i64, List::cons(-25i64, List::<i64>::nil()))))), List::cons(List::cons(86i64, List::cons(4i64, List::cons(-31i64, List::cons(-70i64, List::cons(-5i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-102i64, List::cons(-123i64, List::cons(-10i64, List::cons(-9i64, List::cons(-47i64, List::<i64>::nil()))))), List::cons(List::cons(-83i64, List::cons(-43i64, List::cons(18i64, List::cons(2i64, List::cons(-62i64, List::<i64>::nil()))))), List::cons(List::cons(-15i64, List::cons(10i64, List::cons(15i64, List::cons(-26i64, List::cons(-22i64, List::<i64>::nil()))))), List::cons(List::cons(-42i64, List::cons(13i64, List::cons(12i64, List::cons(-38i64, List::cons(-6i64, List::<i64>::nil()))))), List::cons(List::cons(-26i64, List::cons(10i64, List::cons(-33i64, List::cons(4i64, List::cons(-54i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-67i64, List::cons(-92i64, List::cons(-93i64, List::cons(-29i64, List::cons(4i64, List::<i64>::nil()))))), List::cons(List::cons(-25i64, List::cons(-73i64, List::cons(-8i64, List::cons(-46i64, List::cons(3i64, List::<i64>::nil()))))), List::cons(List::cons(5i64, List::cons(13i64, List::cons(-4i64, List::cons(8i64, List::cons(41i64, List::<i64>::nil()))))), List::cons(List::cons(-51i64, List::cons(-3i64, List::cons(-33i64, List::cons(33i64, List::cons(-20i64, List::<i64>::nil()))))), List::cons(List::cons(-74i64, List::cons(-19i64, List::cons(-33i64, List::cons(-30i64, List::cons(-42i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-35i64, List::cons(-67i64, List::cons(-71i64, List::cons(-26i64, List::cons(-39i64, List::<i64>::nil()))))), List::cons(List::cons(2i64, List::cons(-90i64, List::cons(-4i64, List::cons(10i64, List::cons(-7i64, List::<i64>::nil()))))), List::cons(List::cons(29i64, List::cons(-60i64, List::cons(13i64, List::cons(42i64, List::cons(27i64, List::<i64>::nil()))))), List::cons(List::cons(-55i64, List::cons(43i64, List::cons(49i64, List::cons(39i64, List::cons(3i64, List::<i64>::nil()))))), List::cons(List::cons(-48i64, List::cons(107i64, List::cons(60i64, List::cons(28i64, List::cons(-1i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::cons(List::cons(List::cons(-43i64, List::cons(2i64, List::cons(19i64, List::cons(18i64, List::cons(20i64, List::<i64>::nil()))))), List::cons(List::cons(-91i64, List::cons(-73i64, List::cons(4i64, List::cons(39i64, List::cons(58i64, List::<i64>::nil()))))), List::cons(List::cons(-127i64, List::cons(20i64, List::cons(-19i64, List::cons(-41i64, List::cons(-28i64, List::<i64>::nil()))))), List::cons(List::cons(-30i64, List::cons(-2i64, List::cons(-1i64, List::cons(-61i64, List::cons(-28i64, List::<i64>::nil()))))), List::cons(List::cons(-61i64, List::cons(15i64, List::cons(-41i64, List::cons(4i64, List::cons(-33i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil())))))))), List::<List<List<List<i64>>>>::nil()))))))))))))))))
}

pub fn f50() -> List<i64> {
    List::cons(-6i64, List::cons(-9i64, List::cons(-10i64, List::cons(-17i64, List::cons(-15i64, List::cons(-17i64, List::cons(-9i64, List::cons(-19i64, List::cons(-17i64, List::cons(-20i64, List::cons(-7i64, List::cons(1i64, List::cons(-1i64, List::cons(-25i64, List::cons(-16i64, List::cons(-3i64, List::<i64>::nil()))))))))))))))))
}

pub fn f51() -> List<i64> {
    List::cons(5689854i64, List::cons(5191876i64, List::cons(6405123i64, List::cons(5135996i64, List::cons(5220704i64, List::cons(5631530i64, List::cons(6183396i64, List::cons(4445988i64, List::cons(6704696i64, List::cons(5675925i64, List::cons(4923214i64, List::cons(5922204i64, List::cons(3614080i64, List::cons(7108288i64, List::cons(4709198i64, List::cons(5442370i64, List::<i64>::nil()))))))))))))))))
}

pub fn f52(v0: List<List<List<i64>>>, v1: i64, v2: i64, v3: List<List<i64>>, v4: List<List<i64>>, v5: List<List<i64>>, v6: List<List<i64>>, v7: List<List<i64>>, v8: List<List<i64>>, v9: List<List<i64>>, v10: List<List<i64>>) -> R<List<List<i64>>> {
    let v11: List<i64> = f60(f61(v0.clone(), 0u64));
    let v12: List<i64> = f60(f61(v0.clone(), 1u64));
    let v13: List<i64> = f60(f61(v0.clone(), 2u64));
    let v14: List<i64> = f60(f61(v0.clone(), 3u64));
    let v15: List<i64> = f60(f61(v0.clone(), 4u64));
    let v16: List<i64> = f60(f61(v0.clone(), 5u64));
    let v17: List<i64> = f60(f61(v0.clone(), 6u64));
    let v18: List<i64> = f60(f61(v0, 7u64));
    f71(v1, 0i64, v2, f72(v18, v10, f73(f72(v17, v9, f73(f72(v16, v8, f73(f72(v15, v7, f73(f72(v14, v6, f73(f72(v13, v5, f73(f72(v12, v4, f73(f72(v11, v3, f74())?)?)?)?)?)?)?)?)?)?)?)?)?)?)?)
}

pub fn f53(v0: List<List<List<List<i64>>>>, v1: u64) -> List<List<List<i64>>> {
    let m165 = {
        let a163 = v0;
        let a164 = v1;
        index_list(a163, a164)
    };
    match m165 {
        None => {
            List::<List<List<i64>>>::nil()
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f54(v0: List<List<List<i64>>>, v1: List<List<i64>>) -> R<List<List<List<i64>>>> {
    Ok(List::cons(f75(v1, f76(f77()))?, v0))
}

pub fn f55(v0: List<List<List<i64>>>, v1: List<List<i64>>) -> R<List<List<List<i64>>>> {
    Ok(List::cons(f78(v1)?, v0))
}

pub fn f56() -> List<List<List<List<i64>>>> {
    List::cons(List::cons(List::cons(List::cons(-1i64, List::cons(-30i64, List::cons(-63i64, List::cons(-8i64, List::cons(18i64, List::<i64>::nil()))))), List::cons(List::cons(-74i64, List::cons(-59i64, List::cons(-6i64, List::cons(96i64, List::cons(33i64, List::<i64>::nil()))))), List::cons(List::cons(-61i64, List::cons(7i64, List::cons(127i64, List::cons(69i64, List::cons(-55i64, List::<i64>::nil()))))), List::cons(List::cons(-20i64, List::cons(69i64, List::cons(74i64, List::cons(-37i64, List::cons(-76i64, List::<i64>::nil()))))), List::cons(List::cons(5i64, List::cons(28i64, List::cons(-27i64, List::cons(-59i64, List::cons(-36i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil()), List::cons(List::cons(List::cons(List::cons(-9i64, List::cons(49i64, List::cons(112i64, List::cons(95i64, List::cons(11i64, List::<i64>::nil()))))), List::cons(List::cons(-23i64, List::cons(-62i64, List::cons(-1i64, List::cons(118i64, List::cons(90i64, List::<i64>::nil()))))), List::cons(List::cons(-34i64, List::cons(-73i64, List::cons(-36i64, List::cons(80i64, List::cons(83i64, List::<i64>::nil()))))), List::cons(List::cons(-127i64, List::cons(-86i64, List::cons(-39i64, List::cons(51i64, List::cons(74i64, List::<i64>::nil()))))), List::cons(List::cons(-85i64, List::cons(-51i64, List::cons(-15i64, List::cons(-3i64, List::cons(64i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil()), List::cons(List::cons(List::cons(List::cons(-3i64, List::cons(14i64, List::cons(33i64, List::cons(47i64, List::cons(90i64, List::<i64>::nil()))))), List::cons(List::cons(44i64, List::cons(54i64, List::cons(50i64, List::cons(43i64, List::cons(32i64, List::<i64>::nil()))))), List::cons(List::cons(-61i64, List::cons(-10i64, List::cons(-19i64, List::cons(-23i64, List::cons(-32i64, List::<i64>::nil()))))), List::cons(List::cons(-127i64, List::cons(-93i64, List::cons(-70i64, List::cons(-81i64, List::cons(-59i64, List::<i64>::nil()))))), List::cons(List::cons(-26i64, List::cons(-44i64, List::cons(-46i64, List::cons(-19i64, List::cons(-8i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil()), List::cons(List::cons(List::cons(List::cons(-79i64, List::cons(-89i64, List::cons(-87i64, List::cons(-21i64, List::cons(-59i64, List::<i64>::nil()))))), List::cons(List::cons(-127i64, List::cons(-56i64, List::cons(-15i64, List::cons(44i64, List::cons(43i64, List::<i64>::nil()))))), List::cons(List::cons(-104i64, List::cons(-6i64, List::cons(33i64, List::cons(11i64, List::cons(61i64, List::<i64>::nil()))))), List::cons(List::cons(25i64, List::cons(99i64, List::cons(58i64, List::cons(-3i64, List::cons(48i64, List::<i64>::nil()))))), List::cons(List::cons(87i64, List::cons(94i64, List::cons(8i64, List::cons(56i64, List::cons(91i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil()), List::cons(List::cons(List::cons(List::cons(-40i64, List::cons(-23i64, List::cons(0i64, List::cons(-42i64, List::cons(-30i64, List::<i64>::nil()))))), List::cons(List::cons(17i64, List::cons(38i64, List::cons(-29i64, List::cons(-42i64, List::cons(-104i64, List::<i64>::nil()))))), List::cons(List::cons(-11i64, List::cons(76i64, List::cons(-3i64, List::cons(-88i64, List::cons(-105i64, List::<i64>::nil()))))), List::cons(List::cons(-21i64, List::cons(26i64, List::cons(51i64, List::cons(8i64, List::cons(-2i64, List::<i64>::nil()))))), List::cons(List::cons(2i64, List::cons(46i64, List::cons(116i64, List::cons(127i64, List::cons(44i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil()), List::cons(List::cons(List::cons(List::cons(-29i64, List::cons(-58i64, List::cons(-33i64, List::cons(9i64, List::cons(25i64, List::<i64>::nil()))))), List::cons(List::cons(32i64, List::cons(-95i64, List::cons(-127i64, List::cons(-102i64, List::cons(-67i64, List::<i64>::nil()))))), List::cons(List::cons(94i64, List::cons(60i64, List::cons(-44i64, List::cons(-60i64, List::cons(-37i64, List::<i64>::nil()))))), List::cons(List::cons(38i64, List::cons(114i64, List::cons(63i64, List::cons(22i64, List::cons(-19i64, List::<i64>::nil()))))), List::cons(List::cons(10i64, List::cons(52i64, List::cons(41i64, List::cons(81i64, List::cons(39i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil()), List::cons(List::cons(List::cons(List::cons(22i64, List::cons(127i64, List::cons(-49i64, List::cons(-104i64, List::cons(-48i64, List::<i64>::nil()))))), List::cons(List::cons(83i64, List::cons(111i64, List::cons(-93i64, List::cons(-97i64, List::cons(-56i64, List::<i64>::nil()))))), List::cons(List::cons(94i64, List::cons(68i64, List::cons(-104i64, List::cons(-69i64, List::cons(-20i64, List::<i64>::nil()))))), List::cons(List::cons(63i64, List::cons(26i64, List::cons(-39i64, List::cons(-80i64, List::cons(-35i64, List::<i64>::nil()))))), List::cons(List::cons(79i64, List::cons(42i64, List::cons(16i64, List::cons(30i64, List::cons(-10i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil()), List::cons(List::cons(List::cons(List::cons(-5i64, List::cons(-30i64, List::cons(21i64, List::cons(52i64, List::cons(76i64, List::<i64>::nil()))))), List::cons(List::cons(37i64, List::cons(84i64, List::cons(44i64, List::cons(71i64, List::cons(127i64, List::<i64>::nil()))))), List::cons(List::cons(-26i64, List::cons(44i64, List::cons(30i64, List::cons(11i64, List::cons(30i64, List::<i64>::nil()))))), List::cons(List::cons(6i64, List::cons(-31i64, List::cons(-27i64, List::cons(-44i64, List::cons(-112i64, List::<i64>::nil()))))), List::cons(List::cons(-8i64, List::cons(-43i64, List::cons(-96i64, List::cons(-64i64, List::cons(-43i64, List::<i64>::nil()))))), List::<List<i64>>::nil()))))), List::<List<List<i64>>>::nil()), List::<List<List<List<i64>>>>::nil()))))))))
}

pub fn f57() -> List<i64> {
    List::cons(-20i64, List::cons(-97i64, List::cons(12i64, List::cons(-4i64, List::cons(-12i64, List::cons(-23i64, List::cons(5i64, List::cons(-27i64, List::<i64>::nil()))))))))
}

pub fn f58() -> List<i64> {
    List::cons(9365133i64, List::cons(5217421i64, List::cons(8939750i64, List::cons(4381848i64, List::cons(6279764i64, List::cons(6739462i64, List::cons(5142130i64, List::cons(5212496i64, List::<i64>::nil()))))))))
}

pub fn f59(v0: List<i64>, v1: i64, v2: i64, v3: List<List<i64>>) -> R<List<List<i64>>> {
    let v4: List<List<i64>> = f75(v3, f76(f79()))?;
    f71(v1, 0i64, v2, f72(v0, v4, f80())?)
}

pub fn f60(v0: List<List<i64>>) -> List<i64> {
    f68(List::<i64>::nil(), v0)
}

pub fn f61(v0: List<List<List<i64>>>, v1: u64) -> List<List<i64>> {
    let m168 = {
        let a166 = v0;
        let a167 = v1;
        index_list(a166, a167)
    };
    match m168 {
        None => {
            List::<List<i64>>::nil()
        }
        Some(v2) => {
            v2
        }
    }
}

pub fn f62(v0: Fn7, v1: List<i64>, v2: List<i64>) -> R<List<i64>> {
    let m169 = v2;
    match m169.uncons() {
        None => {
            Ok(v1)
        }
        Some((v3, v4)) => {
            f62(v0.clone(), {
                let c170: Fn7 = v0;
                c170.apply(v1, v3)?
            }, v4)
        }
    }
}

pub fn f63(v0: List<i64>, v1: i64) -> List<i64> {
    List::cons(v1, v0)
}

pub fn f64() -> List<i64> {
    List::cons(112i64, List::cons(4294987264i64, List::cons(74934i64, List::cons(121i64, List::cons(112i64, List::<i64>::nil())))))
}

pub fn f65(v0: Fn8, v1: (List<i64>, i64), v2: List<i64>) -> R<(List<i64>, i64)> {
    let m171 = v2;
    match m171.uncons() {
        None => {
            Ok(v1)
        }
        Some((v3, v4)) => {
            f65(v0.clone(), {
                let c172: Fn8 = v0;
                c172.apply(v1, v3)?
            }, v4)
        }
    }
}

pub fn f66(v0: (List<i64>, i64), v1: i64) -> R<(List<i64>, i64)> {
    let m174 = {
        let (h173, _) = v0;
        h173
    };
    match m174.uncons() {
        None => {
            Ok((List::<i64>::nil(), {
                let (_, h175) = v0;
                h175
            }))
        }
        Some((v2, v3)) => {
            Ok((v3, {
                let a180 = {
                    let (_, h176) = v0;
                    h176
                };
                let a181 = {
                    let a179 = {
                        let a177 = v1;
                        let a178 = v2;
                        int_mul(a177, a178)?
                    };
                    int_neg(a179)?
                };
                int_sub(a180, a181)?
            }))
        }
    }
}

pub fn f67(v0: Bytes, v1: u64) -> R<i64> {
    let m184 = {
        let a182 = v0;
        let a183 = v1;
        index_bytes(a182, a183)
    };
    match m184 {
        None => {
            Ok(0i64)
        }
        Some(v2) => {
            let m187 = {
                let a186 = {
                    let a185 = v2;
                    format_u8(a185)
                };
                parse_int(a186)?
            };
            match m187 {
                None => {
                    Ok(0i64)
                }
                Some(v3) => {
                    Ok(v3)
                }
            }
        }
    }
}

pub fn f68(v0: List<i64>, v1: List<List<i64>>) -> List<i64> {
    f81(Fn9::F82, v0, v1)
}

pub fn f69(v0: List<List<List<i64>>>, v1: List<List<i64>>) -> List<List<List<i64>>> {
    List::cons(v1, v0)
}

pub fn f70(v0: List<List<i64>>) -> R<List<List<i64>>> {
    f8({
        let (_, h188) = f83(Fn10::F84, ((f77(), 0i64), List::<List<i64>>::nil()), v0)?;
        h188
    })
}

pub fn f71(v0: i64, v1: i64, v2: i64, v3: List<List<i64>>) -> R<List<List<i64>>> {
    f23(Fn2::F85(v2, v0, v1), List::<List<i64>>::nil(), v3)
}

pub fn f72(v0: List<i64>, v1: List<List<i64>>, v2: List<List<i64>>) -> R<List<List<i64>>> {
    let m189 = v0;
    match m189.uncons() {
        None => {
            Ok(List::<List<i64>>::nil())
        }
        Some((v3, v4)) => {
            let m190 = v4;
            match m190.uncons() {
                None => {
                    Ok(List::<List<i64>>::nil())
                }
                Some((v5, v6)) => {
                    let m191 = v6;
                    match m191.uncons() {
                        None => {
                            Ok(List::<List<i64>>::nil())
                        }
                        Some((v7, v8)) => {
                            let m192 = v8;
                            match m192.uncons() {
                                None => {
                                    Ok(List::<List<i64>>::nil())
                                }
                                Some((v9, v10)) => {
                                    let m193 = v10;
                                    match m193.uncons() {
                                        None => {
                                            Ok(List::<List<i64>>::nil())
                                        }
                                        Some((v11, v12)) => {
                                            let m194 = v12;
                                            match m194.uncons() {
                                                None => {
                                                    Ok(List::<List<i64>>::nil())
                                                }
                                                Some((v13, v14)) => {
                                                    let m195 = v14;
                                                    match m195.uncons() {
                                                        None => {
                                                            Ok(List::<List<i64>>::nil())
                                                        }
                                                        Some((v15, v16)) => {
                                                            let m196 = v16;
                                                            match m196.uncons() {
                                                                None => {
                                                                    Ok(List::<List<i64>>::nil())
                                                                }
                                                                Some((v17, v18)) => {
                                                                    let m197 = v18;
                                                                    match m197.uncons() {
                                                                        None => {
                                                                            Ok(List::<List<i64>>::nil())
                                                                        }
                                                                        Some((v19, v20)) => {
                                                                            let m198 = v20;
                                                                            match m198.uncons() {
                                                                                None => {
                                                                                    Ok(List::<List<i64>>::nil())
                                                                                }
                                                                                Some((v21, v22)) => {
                                                                                    let m199 = v22;
                                                                                    match m199.uncons() {
                                                                                        None => {
                                                                                            Ok(List::<List<i64>>::nil())
                                                                                        }
                                                                                        Some((v23, v24)) => {
                                                                                            let m200 = v24;
                                                                                            match m200.uncons() {
                                                                                                None => {
                                                                                                    Ok(List::<List<i64>>::nil())
                                                                                                }
                                                                                                Some((v25, v26)) => {
                                                                                                    let m201 = v26;
                                                                                                    match m201.uncons() {
                                                                                                        None => {
                                                                                                            Ok(List::<List<i64>>::nil())
                                                                                                        }
                                                                                                        Some((v27, v28)) => {
                                                                                                            let m202 = v28;
                                                                                                            match m202.uncons() {
                                                                                                                None => {
                                                                                                                    Ok(List::<List<i64>>::nil())
                                                                                                                }
                                                                                                                Some((v29, v30)) => {
                                                                                                                    let m203 = v30;
                                                                                                                    match m203.uncons() {
                                                                                                                        None => {
                                                                                                                            Ok(List::<List<i64>>::nil())
                                                                                                                        }
                                                                                                                        Some((v31, v32)) => {
                                                                                                                            let m204 = v32;
                                                                                                                            match m204.uncons() {
                                                                                                                                None => {
                                                                                                                                    Ok(List::<List<i64>>::nil())
                                                                                                                                }
                                                                                                                                Some((v33, v34)) => {
                                                                                                                                    let m205 = v34;
                                                                                                                                    match m205.uncons() {
                                                                                                                                        None => {
                                                                                                                                            Ok(List::<List<i64>>::nil())
                                                                                                                                        }
                                                                                                                                        Some((v35, v36)) => {
                                                                                                                                            let m206 = v36;
                                                                                                                                            match m206.uncons() {
                                                                                                                                                None => {
                                                                                                                                                    Ok(List::<List<i64>>::nil())
                                                                                                                                                }
                                                                                                                                                Some((v37, v38)) => {
                                                                                                                                                    let m207 = v38;
                                                                                                                                                    match m207.uncons() {
                                                                                                                                                        None => {
                                                                                                                                                            Ok(List::<List<i64>>::nil())
                                                                                                                                                        }
                                                                                                                                                        Some((v39, v40)) => {
                                                                                                                                                            let m208 = v40;
                                                                                                                                                            match m208.uncons() {
                                                                                                                                                                None => {
                                                                                                                                                                    Ok(List::<List<i64>>::nil())
                                                                                                                                                                }
                                                                                                                                                                Some((v41, v42)) => {
                                                                                                                                                                    let m209 = v42;
                                                                                                                                                                    match m209.uncons() {
                                                                                                                                                                        None => {
                                                                                                                                                                            Ok(List::<List<i64>>::nil())
                                                                                                                                                                        }
                                                                                                                                                                        Some((v43, v44)) => {
                                                                                                                                                                            let m210 = v44;
                                                                                                                                                                            match m210.uncons() {
                                                                                                                                                                                None => {
                                                                                                                                                                                    Ok(List::<List<i64>>::nil())
                                                                                                                                                                                }
                                                                                                                                                                                Some((v45, v46)) => {
                                                                                                                                                                                    let m211 = v46;
                                                                                                                                                                                    match m211.uncons() {
                                                                                                                                                                                        None => {
                                                                                                                                                                                            Ok(List::<List<i64>>::nil())
                                                                                                                                                                                        }
                                                                                                                                                                                        Some((v47, v48)) => {
                                                                                                                                                                                            let m212 = v48;
                                                                                                                                                                                            match m212.uncons() {
                                                                                                                                                                                                None => {
                                                                                                                                                                                                    Ok(List::<List<i64>>::nil())
                                                                                                                                                                                                }
                                                                                                                                                                                                Some((v49, v50)) => {
                                                                                                                                                                                                    let m213 = v50;
                                                                                                                                                                                                    match m213.uncons() {
                                                                                                                                                                                                        None => {
                                                                                                                                                                                                            Ok(List::<List<i64>>::nil())
                                                                                                                                                                                                        }
                                                                                                                                                                                                        Some((v51, _)) => {
                                                                                                                                                                                                            f86(v3, v5, v7, v9, v11, v13, v15, v17, v19, v21, v23, v25, v27, v29, v31, v33, v35, v37, v39, v41, v43, v45, v47, v49, v51, v1, v2)
                                                                                                                                                                                                        }
                                                                                                                                                                                                    }
                                                                                                                                                                                                }
                                                                                                                                                                                            }
                                                                                                                                                                                        }
                                                                                                                                                                                    }
                                                                                                                                                                                }
                                                                                                                                                                            }
                                                                                                                                                                        }
                                                                                                                                                                    }
                                                                                                                                                                }
                                                                                                                                                            }
                                                                                                                                                        }
                                                                                                                                                    }
                                                                                                                                                }
                                                                                                                                            }
                                                                                                                                        }
                                                                                                                                    }
                                                                                                                                }
                                                                                                                            }
                                                                                                                        }
                                                                                                                    }
                                                                                                                }
                                                                                                            }
                                                                                                        }
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                        }
                                                                                    }
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn f73(v0: List<List<i64>>) -> R<List<List<i64>>> {
    f23(Fn2::F87, List::<List<i64>>::nil(), v0)
}

pub fn f74() -> List<List<i64>> {
    List::cons(f77(), List::cons(f77(), List::cons(f77(), List::cons(f77(), List::cons(f77(), List::cons(f77(), List::cons(f77(), List::cons(f77(), List::cons(f77(), List::cons(f77(), List::cons(f77(), List::cons(f77(), List::cons(f77(), List::cons(f77(), List::<List<i64>>::nil()))))))))))))))
}

pub fn f75(v0: List<List<i64>>, v1: List<i64>) -> R<List<List<i64>>> {
    let a214 = List::cons(v1.clone(), List::cons(v1.clone(), f8(f23(Fn2::F88, List::<List<i64>>::nil(), v0)?)?));
    let a215 = List::cons(v1.clone(), List::cons(v1, List::<List<i64>>::nil()));
    Ok(append_list(a214, a215))
}

pub fn f76(v0: List<i64>) -> List<i64> {
    let a216 = List::cons(0i64, List::cons(0i64, v0));
    let a217 = List::cons(0i64, List::cons(0i64, List::<i64>::nil()));
    append_list(a216, a217)
}

pub fn f77() -> List<i64> {
    List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::<i64>::nil()))))))))))))))
}

pub fn f78(v0: List<List<i64>>) -> R<List<List<i64>>> {
    f8({
        let (_, h218) = f83(Fn10::F89, ((List::<i64>::nil(), 0i64), List::<List<i64>>::nil()), v0)?;
        h218
    })
}

pub fn f79() -> List<i64> {
    List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::cons(0i64, List::<i64>::nil()))))))))))))))))))))))))))))
}

pub fn f80() -> List<List<i64>> {
    List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::cons(f79(), List::<List<i64>>::nil()))))))))))))))))))))))))))))
}

pub fn f81(v0: Fn9, v1: List<i64>, v2: List<List<i64>>) -> List<i64> {
    let m219 = v2;
    match m219.uncons() {
        None => {
            v1
        }
        Some((v3, v4)) => {
            f81(v0.clone(), {
                let c220: Fn9 = v0;
                c220.apply(v1, v3)
            }, v4)
        }
    }
}

pub fn f82(v0: List<i64>, v1: List<i64>) -> List<i64> {
    let a221 = v0;
    let a222 = v1;
    append_list(a221, a222)
}

pub fn f83(v0: Fn10, v1: ((List<i64>, i64), List<List<i64>>), v2: List<List<i64>>) -> R<((List<i64>, i64), List<List<i64>>)> {
    let m223 = v2;
    match m223.uncons() {
        None => {
            Ok(v1)
        }
        Some((v3, v4)) => {
            f83(v0.clone(), {
                let c224: Fn10 = v0;
                c224.apply(v1, v3)?
            }, v4)
        }
    }
}

pub fn f84(v0: ((List<i64>, i64), List<List<i64>>), v1: List<i64>) -> R<((List<i64>, i64), List<List<i64>>)> {
    let v2: List<i64> = {
        let (h225, _) = {
            let (h226, _) = v0.clone();
            h226
        };
        h225
    };
    let v3: i64 = {
        let (_, h227) = {
            let (h228, _) = v0;
            h228
        };
        h227
    };
    let v4: List<i64> = f90(v2, v1)?;
    let v5: bool = {
        let m231 = {
            let a229 = 1i64;
            let a230 = v3;
            compare(a229, a230)
        };
        match m231 {
            Ordering::Less => {
                true
            }
            Ordering::Equal => {
                false
            }
            Ordering::Greater => {
                false
            }
        }
    };
    let v6: List<List<i64>> = if v5 {
        List::cons(f91(v4.clone())?, {
            let (_, h232) = v0;
            h232
        })
    } else {
        let (_, h233) = v0;
        h233
    };
    let v7: List<i64> = if v5 {
        f77()
    } else {
        v4
    };
    let v8: i64 = if v5 {
        0i64
    } else {
        let a235 = v3;
        let a236 = {
            let a234 = 1i64;
            int_neg(a234)?
        };
        int_sub(a235, a236)?
    };
    Ok(((v7, v8), v6))
}

pub fn f85(v0: i64, v1: i64, v2: i64, v3: List<List<i64>>, v4: List<i64>) -> R<List<List<i64>>> {
    Ok(List::cons(f92(v1, v2, v0, v4)?, v3))
}

pub fn f86(v0: i64, v1: i64, v2: i64, v3: i64, v4: i64, v5: i64, v6: i64, v7: i64, v8: i64, v9: i64, v10: i64, v11: i64, v12: i64, v13: i64, v14: i64, v15: i64, v16: i64, v17: i64, v18: i64, v19: i64, v20: i64, v21: i64, v22: i64, v23: i64, v24: i64, v25: List<List<i64>>, v26: List<List<i64>>) -> R<List<List<i64>>> {
    let (_, h237) = {
        let (_, h238) = {
            let (_, h239) = f93(Fn11::F94(v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15, v16, v17, v18, v19, v20, v21, v22, v23, v24), ((List::<i64>::nil(), (List::<i64>::nil(), List::<i64>::nil())), ((List::<i64>::nil(), v26), (0i64, List::<List<i64>>::nil()))), v25)?;
            h239
        };
        h238
    };
    Ok(h237)
}

pub fn f87(v0: List<List<i64>>, v1: List<i64>) -> R<List<List<i64>>> {
    Ok(List::cons(f34(v1)?, v0))
}

pub fn f88(v0: List<List<i64>>, v1: List<i64>) -> List<List<i64>> {
    List::cons(f76(v1), v0)
}

pub fn f89(v0: ((List<i64>, i64), List<List<i64>>), v1: List<i64>) -> R<((List<i64>, i64), List<List<i64>>)> {
    let v2: List<i64> = {
        let (h240, _) = {
            let (h241, _) = v0.clone();
            h241
        };
        h240
    };
    let v3: i64 = {
        let (_, h242) = {
            let (h243, _) = v0;
            h243
        };
        h242
    };
    let v4: List<i64> = f95(f90(v2, v1.clone())?)?;
    let v5: List<List<i64>> = {
        let m246 = {
            let a244 = 0i64;
            let a245 = v3;
            compare(a244, a245)
        };
        if match m246 {
            Ordering::Less => {
                true
            }
            Ordering::Equal => {
                false
            }
            Ordering::Greater => {
                false
            }
        } {
            List::cons(v4, {
                let (_, h247) = v0;
                h247
            })
        } else {
            let (_, h248) = v0;
            h248
        }
    };
    Ok(((v1, {
        let a249 = 1i64;
        let a250 = v3;
        int_sub(a249, a250)?
    }), v5))
}

pub fn f90(v0: List<i64>, v1: List<i64>) -> R<List<i64>> {
    f34({
        let (_, h251) = f96(Fn12::F97, (v1, List::<i64>::nil()), v0);
        h251
    })
}

pub fn f91(v0: List<i64>) -> R<List<i64>> {
    f34({
        let (_, h252) = f98(Fn13::F99, ((0i64, 0i64), List::<i64>::nil()), v0)?;
        h252
    })
}

pub fn f92(v0: i64, v1: i64, v2: i64, v3: List<i64>) -> R<List<i64>> {
    f62(Fn7::F100(v2, v0, v1), List::<i64>::nil(), v3)
}

pub fn f93(v0: Fn11, v1: ((List<i64>, (List<i64>, List<i64>)), ((List<i64>, List<List<i64>>), (i64, List<List<i64>>))), v2: List<List<i64>>) -> R<((List<i64>, (List<i64>, List<i64>)), ((List<i64>, List<List<i64>>), (i64, List<List<i64>>)))> {
    let m253 = v2;
    match m253.uncons() {
        None => {
            Ok(v1)
        }
        Some((v3, v4)) => {
            f93(v0.clone(), {
                let c254: Fn11 = v0;
                c254.apply(v1, v3)?
            }, v4)
        }
    }
}

pub fn f94(v0: i64, v1: i64, v2: i64, v3: i64, v4: i64, v5: i64, v6: i64, v7: i64, v8: i64, v9: i64, v10: i64, v11: i64, v12: i64, v13: i64, v14: i64, v15: i64, v16: i64, v17: i64, v18: i64, v19: i64, v20: i64, v21: i64, v22: i64, v23: i64, v24: i64, v25: ((List<i64>, (List<i64>, List<i64>)), ((List<i64>, List<List<i64>>), (i64, List<List<i64>>))), v26: List<i64>) -> R<((List<i64>, (List<i64>, List<i64>)), ((List<i64>, List<List<i64>>), (i64, List<List<i64>>)))> {
    let v27: (List<i64>, (List<i64>, List<i64>)) = {
        let (h255, _) = v25;
        h255
    };
    let v28: (List<i64>, List<i64>) = {
        let (_, h256) = v27;
        h256
    };
    let v29: ((List<i64>, List<List<i64>>), (i64, List<List<i64>>)) = {
        let (_, h257) = v25;
        h257
    };
    let v30: (List<i64>, List<List<i64>>) = {
        let (h258, _) = v29;
        h258
    };
    let v31: (i64, List<List<i64>>) = {
        let (_, h259) = v29;
        h259
    };
    let v34: (List<i64>, List<List<i64>>) = {
        let m261 = {
            let (_, h260) = v30.clone();
            h260
        };
        match m261.uncons() {
            None => {
                (List::<i64>::nil(), List::<List<i64>>::nil())
            }
            Some((v32, v33)) => {
                (v32, v33)
            }
        }
    };
    let v35: List<i64> = {
        let (h262, _) = v34;
        h262
    };
    let v36: List<List<i64>> = {
        let (_, h263) = v34;
        h263
    };
    let v37: bool = {
        let m267 = {
            let a265 = 3i64;
            let a266 = {
                let (h264, _) = v31.clone();
                h264
            };
            compare(a265, a266)
        };
        match m267 {
            Ordering::Less => {
                true
            }
            Ordering::Equal => {
                false
            }
            Ordering::Greater => {
                false
            }
        }
    };
    let v38: List<i64> = f101(v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15, v16, v17, v18, v19, v20, v21, v22, v23, v24, v35, {
        let (h268, _) = v27;
        h268
    }, {
        let (h269, _) = v28.clone();
        h269
    }, {
        let (_, h270) = v28.clone();
        h270
    }, {
        let (h271, _) = v30.clone();
        h271
    }, v26.clone())?;
    let v39: List<List<i64>> = if v37 {
        List::cons(v38, {
            let (_, h272) = v31;
            h272
        })
    } else {
        let (_, h273) = v31;
        h273
    };
    let v40: List<List<i64>> = if v37 {
        v36
    } else {
        let (_, h274) = v30;
        h274
    };
    Ok((({
        let (h275, _) = v28;
        h275
    }, ({
        let (_, h276) = v28;
        h276
    }, {
        let (h277, _) = v30;
        h277
    })), ((v26, v40), ({
        let a280 = {
            let (h278, _) = v31;
            h278
        };
        let a281 = {
            let a279 = 1i64;
            int_neg(a279)?
        };
        int_sub(a280, a281)?
    }, v39))))
}

pub fn f95(v0: List<i64>) -> R<List<i64>> {
    f34({
        let (_, h282) = f98(Fn13::F102, ((0i64, 0i64), List::<i64>::nil()), v0)?;
        h282
    })
}

pub fn f96(v0: Fn12, v1: (List<i64>, List<i64>), v2: List<i64>) -> (List<i64>, List<i64>) {
    let m283 = v2;
    match m283.uncons() {
        None => {
            v1
        }
        Some((v3, v4)) => {
            f96(v0.clone(), {
                let c284: Fn12 = v0;
                c284.apply(v1, v3)
            }, v4)
        }
    }
}

pub fn f97(v0: (List<i64>, List<i64>), v1: i64) -> (List<i64>, List<i64>) {
    let m286 = {
        let (h285, _) = v0;
        h285
    };
    match m286.uncons() {
        None => {
            (List::<i64>::nil(), List::cons(v1, {
                let (_, h287) = v0;
                h287
            }))
        }
        Some((v2, v3)) => {
            (v3, List::cons({
                let m290 = {
                    let a288 = v1;
                    let a289 = v2;
                    compare(a288, a289)
                };
                if match m290 {
                    Ordering::Less => {
                        true
                    }
                    Ordering::Equal => {
                        false
                    }
                    Ordering::Greater => {
                        false
                    }
                } {
                    v2
                } else {
                    v1
                }
            }, {
                let (_, h291) = v0;
                h291
            }))
        }
    }
}

pub fn f98(v0: Fn13, v1: ((i64, i64), List<i64>), v2: List<i64>) -> R<((i64, i64), List<i64>)> {
    let m292 = v2;
    match m292.uncons() {
        None => {
            Ok(v1)
        }
        Some((v3, v4)) => {
            f98(v0.clone(), {
                let c293: Fn13 = v0;
                c293.apply(v1, v3)?
            }, v4)
        }
    }
}

pub fn f99(v0: ((i64, i64), List<i64>), v1: i64) -> R<((i64, i64), List<i64>)> {
    let v2: i64 = {
        let (h294, _) = {
            let (h295, _) = v0.clone();
            h295
        };
        h294
    };
    let v3: i64 = {
        let (_, h296) = {
            let (h297, _) = v0;
            h297
        };
        h296
    };
    let v4: i64 = {
        let m300 = {
            let a298 = v2;
            let a299 = v1;
            compare(a298, a299)
        };
        if match m300 {
            Ordering::Less => {
                true
            }
            Ordering::Equal => {
                false
            }
            Ordering::Greater => {
                false
            }
        } {
            v1
        } else {
            v2
        }
    };
    let v5: bool = {
        let m303 = {
            let a301 = 1i64;
            let a302 = v3;
            compare(a301, a302)
        };
        match m303 {
            Ordering::Less => {
                true
            }
            Ordering::Equal => {
                false
            }
            Ordering::Greater => {
                false
            }
        }
    };
    let v6: List<i64> = if v5 {
        List::cons(v4, {
            let (_, h304) = v0;
            h304
        })
    } else {
        let (_, h305) = v0;
        h305
    };
    let v7: i64 = if v5 {
        0i64
    } else {
        v4
    };
    let v8: i64 = if v5 {
        0i64
    } else {
        let a307 = v3;
        let a308 = {
            let a306 = 1i64;
            int_neg(a306)?
        };
        int_sub(a307, a308)?
    };
    Ok(((v7, v8), v6))
}

pub fn f100(v0: i64, v1: i64, v2: i64, v3: List<i64>, v4: i64) -> R<List<i64>> {
    Ok(List::cons(f35(v1, v2, v0, v4)?, v3))
}

pub fn f101(v0: i64, v1: i64, v2: i64, v3: i64, v4: i64, v5: i64, v6: i64, v7: i64, v8: i64, v9: i64, v10: i64, v11: i64, v12: i64, v13: i64, v14: i64, v15: i64, v16: i64, v17: i64, v18: i64, v19: i64, v20: i64, v21: i64, v22: i64, v23: i64, v24: i64, v25: List<i64>, v26: List<i64>, v27: List<i64>, v28: List<i64>, v29: List<i64>, v30: List<i64>) -> R<List<i64>> {
    let (_, h309) = {
        let (_, h310) = {
            let (_, h311) = {
                let (_, h312) = {
                    let (_, h313) = f103(Fn14::F104(v0, v1, v2, v3, v4, v5, v6, v7, v8, v9, v10, v11, v12, v13, v14, v15, v16, v17, v18, v19, v20, v21, v22, v23, v24), ((((v27, (v28, v29)), (v30, (v25, 0i64))), ((0i64, (0i64, 0i64)), ((0i64, 0i64), (0i64, 0i64)))), (((0i64, (0i64, 0i64)), ((0i64, 0i64), (0i64, 0i64))), ((0i64, (0i64, 0i64)), ((0i64, 0i64), (0i64, List::<i64>::nil()))))), v26)?;
                    h313
                };
                h312
            };
            h311
        };
        h310
    };
    Ok(h309)
}

pub fn f102(v0: ((i64, i64), List<i64>), v1: i64) -> R<((i64, i64), List<i64>)> {
    let v2: i64 = {
        let (h314, _) = {
            let (h315, _) = v0.clone();
            h315
        };
        h314
    };
    let v3: i64 = {
        let (_, h316) = {
            let (h317, _) = v0;
            h317
        };
        h316
    };
    let v4: i64 = {
        let m320 = {
            let a318 = v2;
            let a319 = v1;
            compare(a318, a319)
        };
        if match m320 {
            Ordering::Less => {
                true
            }
            Ordering::Equal => {
                false
            }
            Ordering::Greater => {
                false
            }
        } {
            v1
        } else {
            v2
        }
    };
    let v5: List<i64> = {
        let m323 = {
            let a321 = 0i64;
            let a322 = v3;
            compare(a321, a322)
        };
        if match m323 {
            Ordering::Less => {
                true
            }
            Ordering::Equal => {
                false
            }
            Ordering::Greater => {
                false
            }
        } {
            List::cons(v4, {
                let (_, h324) = v0;
                h324
            })
        } else {
            let (_, h325) = v0;
            h325
        }
    };
    Ok(((v1, {
        let a326 = 1i64;
        let a327 = v3;
        int_sub(a326, a327)?
    }), v5))
}

pub fn f103(v0: Fn14, v1: ((((List<i64>, (List<i64>, List<i64>)), (List<i64>, (List<i64>, i64))), ((i64, (i64, i64)), ((i64, i64), (i64, i64)))), (((i64, (i64, i64)), ((i64, i64), (i64, i64))), ((i64, (i64, i64)), ((i64, i64), (i64, List<i64>))))), v2: List<i64>) -> R<((((List<i64>, (List<i64>, List<i64>)), (List<i64>, (List<i64>, i64))), ((i64, (i64, i64)), ((i64, i64), (i64, i64)))), (((i64, (i64, i64)), ((i64, i64), (i64, i64))), ((i64, (i64, i64)), ((i64, i64), (i64, List<i64>)))))> {
    let m328 = v2;
    match m328.uncons() {
        None => {
            Ok(v1)
        }
        Some((v3, v4)) => {
            f103(v0.clone(), {
                let c329: Fn14 = v0;
                c329.apply(v1, v3)?
            }, v4)
        }
    }
}

pub fn f104(v0: i64, v1: i64, v2: i64, v3: i64, v4: i64, v5: i64, v6: i64, v7: i64, v8: i64, v9: i64, v10: i64, v11: i64, v12: i64, v13: i64, v14: i64, v15: i64, v16: i64, v17: i64, v18: i64, v19: i64, v20: i64, v21: i64, v22: i64, v23: i64, v24: i64, v25: ((((List<i64>, (List<i64>, List<i64>)), (List<i64>, (List<i64>, i64))), ((i64, (i64, i64)), ((i64, i64), (i64, i64)))), (((i64, (i64, i64)), ((i64, i64), (i64, i64))), ((i64, (i64, i64)), ((i64, i64), (i64, List<i64>))))), v26: i64) -> R<((((List<i64>, (List<i64>, List<i64>)), (List<i64>, (List<i64>, i64))), ((i64, (i64, i64)), ((i64, i64), (i64, i64)))), (((i64, (i64, i64)), ((i64, i64), (i64, i64))), ((i64, (i64, i64)), ((i64, i64), (i64, List<i64>)))))> {
    let v27: (((List<i64>, (List<i64>, List<i64>)), (List<i64>, (List<i64>, i64))), ((i64, (i64, i64)), ((i64, i64), (i64, i64)))) = {
        let (h330, _) = v25;
        h330
    };
    let v28: ((List<i64>, (List<i64>, List<i64>)), (List<i64>, (List<i64>, i64))) = {
        let (h331, _) = v27;
        h331
    };
    let v29: (List<i64>, (List<i64>, List<i64>)) = {
        let (h332, _) = v28;
        h332
    };
    let v30: (List<i64>, List<i64>) = {
        let (_, h333) = v29;
        h333
    };
    let v31: (List<i64>, (List<i64>, i64)) = {
        let (_, h334) = v28;
        h334
    };
    let v32: (List<i64>, i64) = {
        let (_, h335) = v31;
        h335
    };
    let v33: ((i64, (i64, i64)), ((i64, i64), (i64, i64))) = {
        let (_, h336) = v27;
        h336
    };
    let v34: (i64, (i64, i64)) = {
        let (h337, _) = v33;
        h337
    };
    let v35: (i64, i64) = {
        let (_, h338) = v34;
        h338
    };
    let v36: ((i64, i64), (i64, i64)) = {
        let (_, h339) = v33;
        h339
    };
    let v37: (i64, i64) = {
        let (h340, _) = v36;
        h340
    };
    let v38: (i64, i64) = {
        let (_, h341) = v36;
        h341
    };
    let v39: (((i64, (i64, i64)), ((i64, i64), (i64, i64))), ((i64, (i64, i64)), ((i64, i64), (i64, List<i64>)))) = {
        let (_, h342) = v25;
        h342
    };
    let v40: ((i64, (i64, i64)), ((i64, i64), (i64, i64))) = {
        let (h343, _) = v39;
        h343
    };
    let v41: (i64, (i64, i64)) = {
        let (h344, _) = v40;
        h344
    };
    let v42: (i64, i64) = {
        let (_, h345) = v41;
        h345
    };
    let v43: ((i64, i64), (i64, i64)) = {
        let (_, h346) = v40;
        h346
    };
    let v44: (i64, i64) = {
        let (h347, _) = v43;
        h347
    };
    let v45: (i64, i64) = {
        let (_, h348) = v43;
        h348
    };
    let v46: ((i64, (i64, i64)), ((i64, i64), (i64, List<i64>))) = {
        let (_, h349) = v39;
        h349
    };
    let v47: (i64, (i64, i64)) = {
        let (h350, _) = v46;
        h350
    };
    let v48: (i64, i64) = {
        let (_, h351) = v47;
        h351
    };
    let v49: ((i64, i64), (i64, List<i64>)) = {
        let (_, h352) = v46;
        h352
    };
    let v50: (i64, i64) = {
        let (h353, _) = v49;
        h353
    };
    let v51: (i64, List<i64>) = {
        let (_, h354) = v49;
        h354
    };
    let v54: (i64, List<i64>) = {
        let m356 = {
            let (h355, _) = v29;
            h355
        };
        match m356.uncons() {
            None => {
                (0i64, List::<i64>::nil())
            }
            Some((v52, v53)) => {
                (v52, v53)
            }
        }
    };
    let v55: i64 = {
        let (h357, _) = v54;
        h357
    };
    let v56: List<i64> = {
        let (_, h358) = v54;
        h358
    };
    let v59: (i64, List<i64>) = {
        let m360 = {
            let (h359, _) = v30;
            h359
        };
        match m360.uncons() {
            None => {
                (0i64, List::<i64>::nil())
            }
            Some((v57, v58)) => {
                (v57, v58)
            }
        }
    };
    let v60: i64 = {
        let (h361, _) = v59;
        h361
    };
    let v61: List<i64> = {
        let (_, h362) = v59;
        h362
    };
    let v64: (i64, List<i64>) = {
        let m364 = {
            let (_, h363) = v30;
            h363
        };
        match m364.uncons() {
            None => {
                (0i64, List::<i64>::nil())
            }
            Some((v62, v63)) => {
                (v62, v63)
            }
        }
    };
    let v65: i64 = {
        let (h365, _) = v64;
        h365
    };
    let v66: List<i64> = {
        let (_, h366) = v64;
        h366
    };
    let v69: (i64, List<i64>) = {
        let m368 = {
            let (h367, _) = v31;
            h367
        };
        match m368.uncons() {
            None => {
                (0i64, List::<i64>::nil())
            }
            Some((v67, v68)) => {
                (v67, v68)
            }
        }
    };
    let v70: i64 = {
        let (h369, _) = v69;
        h369
    };
    let v71: List<i64> = {
        let (_, h370) = v69;
        h370
    };
    let v74: (i64, List<i64>) = {
        let m372 = {
            let (h371, _) = v32.clone();
            h371
        };
        match m372.uncons() {
            None => {
                (0i64, List::<i64>::nil())
            }
            Some((v72, v73)) => {
                (v72, v73)
            }
        }
    };
    let v75: i64 = {
        let (h373, _) = v74;
        h373
    };
    let v76: List<i64> = {
        let (_, h374) = v74;
        h374
    };
    let v77: bool = {
        let m378 = {
            let a376 = 3i64;
            let a377 = {
                let (h375, _) = v51.clone();
                h375
            };
            compare(a376, a377)
        };
        match m378 {
            Ordering::Less => {
                true
            }
            Ordering::Equal => {
                false
            }
            Ordering::Greater => {
                false
            }
        }
    };
    let v78: i64 = {
        let a522 = v75;
        let a523 = {
            let a521 = {
                let a519 = {
                    let a444 = {
                        let a409 = {
                            let a392 = {
                                let a380 = v0;
                                let a381 = {
                                    let (_, h379) = v32;
                                    h379
                                };
                                int_mul(a380, a381)?
                            };
                            let a393 = {
                                let a391 = {
                                    let a389 = {
                                        let a383 = v1;
                                        let a384 = {
                                            let (h382, _) = v34;
                                            h382
                                        };
                                        int_mul(a383, a384)?
                                    };
                                    let a390 = {
                                        let a388 = {
                                            let a386 = v2;
                                            let a387 = {
                                                let (h385, _) = v35;
                                                h385
                                            };
                                            int_mul(a386, a387)?
                                        };
                                        int_neg(a388)?
                                    };
                                    int_sub(a389, a390)?
                                };
                                int_neg(a391)?
                            };
                            int_sub(a392, a393)?
                        };
                        let a410 = {
                            let a408 = {
                                let a406 = {
                                    let a395 = v3;
                                    let a396 = {
                                        let (_, h394) = v35;
                                        h394
                                    };
                                    int_mul(a395, a396)?
                                };
                                let a407 = {
                                    let a405 = {
                                        let a403 = {
                                            let a397 = v4;
                                            let a398 = v26;
                                            int_mul(a397, a398)?
                                        };
                                        let a404 = {
                                            let a402 = {
                                                let a400 = v5;
                                                let a401 = {
                                                    let (h399, _) = v37;
                                                    h399
                                                };
                                                int_mul(a400, a401)?
                                            };
                                            int_neg(a402)?
                                        };
                                        int_sub(a403, a404)?
                                    };
                                    int_neg(a405)?
                                };
                                int_sub(a406, a407)?
                            };
                            int_neg(a408)?
                        };
                        int_sub(a409, a410)?
                    };
                    let a445 = {
                        let a443 = {
                            let a441 = {
                                let a424 = {
                                    let a412 = v6;
                                    let a413 = {
                                        let (_, h411) = v37;
                                        h411
                                    };
                                    int_mul(a412, a413)?
                                };
                                let a425 = {
                                    let a423 = {
                                        let a421 = {
                                            let a415 = v7;
                                            let a416 = {
                                                let (h414, _) = v38;
                                                h414
                                            };
                                            int_mul(a415, a416)?
                                        };
                                        let a422 = {
                                            let a420 = {
                                                let a418 = v8;
                                                let a419 = {
                                                    let (_, h417) = v38;
                                                    h417
                                                };
                                                int_mul(a418, a419)?
                                            };
                                            int_neg(a420)?
                                        };
                                        int_sub(a421, a422)?
                                    };
                                    int_neg(a423)?
                                };
                                int_sub(a424, a425)?
                            };
                            let a442 = {
                                let a440 = {
                                    let a438 = {
                                        let a426 = v9;
                                        let a427 = v55;
                                        int_mul(a426, a427)?
                                    };
                                    let a439 = {
                                        let a437 = {
                                            let a435 = {
                                                let a429 = v10;
                                                let a430 = {
                                                    let (h428, _) = v41;
                                                    h428
                                                };
                                                int_mul(a429, a430)?
                                            };
                                            let a436 = {
                                                let a434 = {
                                                    let a432 = v11;
                                                    let a433 = {
                                                        let (h431, _) = v42;
                                                        h431
                                                    };
                                                    int_mul(a432, a433)?
                                                };
                                                int_neg(a434)?
                                            };
                                            int_sub(a435, a436)?
                                        };
                                        int_neg(a437)?
                                    };
                                    int_sub(a438, a439)?
                                };
                                int_neg(a440)?
                            };
                            int_sub(a441, a442)?
                        };
                        int_neg(a443)?
                    };
                    int_sub(a444, a445)?
                };
                let a520 = {
                    let a518 = {
                        let a516 = {
                            let a476 = {
                                let a458 = {
                                    let a447 = v12;
                                    let a448 = {
                                        let (_, h446) = v42;
                                        h446
                                    };
                                    int_mul(a447, a448)?
                                };
                                let a459 = {
                                    let a457 = {
                                        let a455 = {
                                            let a450 = v13;
                                            let a451 = {
                                                let (h449, _) = v44;
                                                h449
                                            };
                                            int_mul(a450, a451)?
                                        };
                                        let a456 = {
                                            let a454 = {
                                                let a452 = v14;
                                                let a453 = v60;
                                                int_mul(a452, a453)?
                                            };
                                            int_neg(a454)?
                                        };
                                        int_sub(a455, a456)?
                                    };
                                    int_neg(a457)?
                                };
                                int_sub(a458, a459)?
                            };
                            let a477 = {
                                let a475 = {
                                    let a473 = {
                                        let a461 = v15;
                                        let a462 = {
                                            let (_, h460) = v44;
                                            h460
                                        };
                                        int_mul(a461, a462)?
                                    };
                                    let a474 = {
                                        let a472 = {
                                            let a470 = {
                                                let a464 = v16;
                                                let a465 = {
                                                    let (h463, _) = v45;
                                                    h463
                                                };
                                                int_mul(a464, a465)?
                                            };
                                            let a471 = {
                                                let a469 = {
                                                    let a467 = v17;
                                                    let a468 = {
                                                        let (_, h466) = v45;
                                                        h466
                                                    };
                                                    int_mul(a467, a468)?
                                                };
                                                int_neg(a469)?
                                            };
                                            int_sub(a470, a471)?
                                        };
                                        int_neg(a472)?
                                    };
                                    int_sub(a473, a474)?
                                };
                                int_neg(a475)?
                            };
                            int_sub(a476, a477)?
                        };
                        let a517 = {
                            let a515 = {
                                let a513 = {
                                    let a490 = {
                                        let a479 = v18;
                                        let a480 = {
                                            let (h478, _) = v47;
                                            h478
                                        };
                                        int_mul(a479, a480)?
                                    };
                                    let a491 = {
                                        let a489 = {
                                            let a487 = {
                                                let a481 = v19;
                                                let a482 = v65;
                                                int_mul(a481, a482)?
                                            };
                                            let a488 = {
                                                let a486 = {
                                                    let a484 = v20;
                                                    let a485 = {
                                                        let (h483, _) = v48;
                                                        h483
                                                    };
                                                    int_mul(a484, a485)?
                                                };
                                                int_neg(a486)?
                                            };
                                            int_sub(a487, a488)?
                                        };
                                        int_neg(a489)?
                                    };
                                    int_sub(a490, a491)?
                                };
                                let a514 = {
                                    let a512 = {
                                        let a510 = {
                                            let a499 = {
                                                let a493 = v21;
                                                let a494 = {
                                                    let (_, h492) = v48;
                                                    h492
                                                };
                                                int_mul(a493, a494)?
                                            };
                                            let a500 = {
                                                let a498 = {
                                                    let a496 = v22;
                                                    let a497 = {
                                                        let (h495, _) = v50;
                                                        h495
                                                    };
                                                    int_mul(a496, a497)?
                                                };
                                                int_neg(a498)?
                                            };
                                            int_sub(a499, a500)?
                                        };
                                        let a511 = {
                                            let a509 = {
                                                let a507 = {
                                                    let a502 = v23;
                                                    let a503 = {
                                                        let (_, h501) = v50;
                                                        h501
                                                    };
                                                    int_mul(a502, a503)?
                                                };
                                                let a508 = {
                                                    let a506 = {
                                                        let a504 = v24;
                                                        let a505 = v70;
                                                        int_mul(a504, a505)?
                                                    };
                                                    int_neg(a506)?
                                                };
                                                int_sub(a507, a508)?
                                            };
                                            int_neg(a509)?
                                        };
                                        int_sub(a510, a511)?
                                    };
                                    int_neg(a512)?
                                };
                                int_sub(a513, a514)?
                            };
                            int_neg(a515)?
                        };
                        int_sub(a516, a517)?
                    };
                    int_neg(a518)?
                };
                int_sub(a519, a520)?
            };
            int_neg(a521)?
        };
        int_sub(a522, a523)?
    };
    let v79: List<i64> = if v77 {
        List::cons(v78, {
            let (_, h524) = v51;
            h524
        })
    } else {
        let (_, h525) = v51;
        h525
    };
    let v80: List<i64> = if v77 {
        v76
    } else {
        let (h526, _) = v32;
        h526
    };
    Ok(((((v56, (v61, v66)), (v71, (v80, {
        let (h527, _) = v34;
        h527
    }))), (({
        let (h528, _) = v35;
        h528
    }, ({
        let (_, h529) = v35;
        h529
    }, v26)), (({
        let (_, h530) = v37;
        h530
    }, {
        let (h531, _) = v38;
        h531
    }), ({
        let (_, h532) = v38;
        h532
    }, v55)))), ((({
        let (h533, _) = v42;
        h533
    }, ({
        let (_, h534) = v42;
        h534
    }, {
        let (h535, _) = v44;
        h535
    })), ((v60, {
        let (h536, _) = v45;
        h536
    }), ({
        let (_, h537) = v45;
        h537
    }, {
        let (h538, _) = v47;
        h538
    }))), ((v65, ({
        let (_, h539) = v48;
        h539
    }, {
        let (h540, _) = v50;
        h540
    })), (({
        let (_, h541) = v50;
        h541
    }, v70), ({
        let a544 = {
            let (h542, _) = v51;
            h542
        };
        let a545 = {
            let a543 = 1i64;
            int_neg(a543)?
        };
        int_sub(a544, a545)?
    }, v79))))))
}
