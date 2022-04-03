#![allow(dead_code)]

use core::marker::PhantomData;
use std::rc::Rc;

pub trait Observer<T: Clone, E> {
  fn start(&self, subscription: dyn Subscription);
  fn next(&self, value: T);
  fn complete(&self);
  fn error(&self, error: E);
}

pub trait Subscription {
  fn unsubscribe(&mut self);
}

pub trait Observable {
  type Item: Clone;
  type Error: Clone;
  type Observer: Observer<Self::Item, Self::Error>;
  type SubscriptionType: Subscription;

  fn subscribe(self: Rc<Self>, observer: Self::Observer) -> Self::SubscriptionType;
  fn unsubscribe(&mut self, observer: Rc<Self::Observer>);
}

struct SimpleObserver<T: Clone, E: Clone> {
  pub start: fn(&mut DefaultSubscription<DefaultObservable<T, E>>),
  pub next: fn(T),
  pub complete: fn(),
  pub error: fn(E),
}

impl<T: Clone, E: Clone> SimpleObserver<T, E> {
  fn new(next: fn(T)) -> Self {
    Self { start: |_subscription| {}, next, complete: || {}, error: |_error| {} }
  }
}

impl<T: Clone, E: Clone> Observer<T, E> for SimpleObserver<T, E> {
  fn start(&self, subscription: &mut DefaultSubscription<DefaultObservable<T, E>>) {
    (self.start)(subscription);
  }
  fn next(&self, value: T) { (self.next)(value); }
  fn complete(&self) { (self.complete)(); }
  fn error(&self, error: E) { (self.error)(error) }
}

impl<T: Clone, E: Clone> Default for SimpleObserver<T, E> {
  fn default() -> SimpleObserver<T, E> {
    SimpleObserver::<T, E> {
      start: |_| {},
      next: |_| {},
      complete: || {},
      error: |_| {},
    }
  }
}

type SubscriptionFunction<T, E> = fn(observer: SimpleObserver<T, E>) -> fn();

struct DefaultSubscription<O: Observable> {
  observable: Rc<O>,
  observer: Rc<O::Observer>,
}

impl<O: Observable> DefaultSubscription<O> {
  fn new(observable: Rc<O>, observer: Rc<O::Observer>) -> Self {
    DefaultSubscription { observable, observer }
  }
}

impl<O: Observable> Subscription for DefaultSubscription<O> {
  fn unsubscribe(&mut self) {
    self.observable.unsubscribe(self.observer);
  }
}

impl<O: Observable> Drop for DefaultSubscription<O> {
  fn drop(&mut self) {
    self.unsubscribe();
  }
}

struct DefaultObservable<T: Clone, E: Clone> {
  _t: PhantomData<(T, E)>,
  subscriber_function: SubscriptionFunction<T, E>,
}

impl<T: Clone, E: Clone> DefaultObservable<T, E> {
  fn new(subscriber_function: SubscriptionFunction<T, E>) -> DefaultObservable<T, E> {
    DefaultObservable {
      _t: Default::default(),
      subscriber_function,
    }
  }
}

impl<T: Clone, E: Clone> Observable for DefaultObservable<T, E> {
  type Item = T;
  type Error = E;
  type Observer = SimpleObserver<T, E>;
  type SubscriptionType = DefaultSubscription<Self>;

  fn subscribe(self: Rc<Self>, observer: Self::Observer) -> Self::SubscriptionType {
    DefaultSubscription::new(self, Rc::new(observer))
  }
  fn unsubscribe(&mut self, _observer: Rc<Self::Observer>) {}
}

fn main() {
  let mut observable = DefaultObservable::<u8, &str>::new(|observer: SimpleObserver<u8, &str>| {
    observer.next(255);
    observer.complete();
    return || {};
  });

  let _subscription1 = observable.subscribe(SimpleObserver::<u8, &str> {
    next: |value: u8| { println!("{:?}", value); },
    complete: || { println!("complete"); },
    error: |error: &str| { println!("error: {:?}", error); },
    ..Default::default()
  });

  let _subscription2 = observable.subscribe(SimpleObserver::<u8, &str> {
    next: |value: u8| { println!("{:?}", value) },
    ..Default::default()
  });
}
