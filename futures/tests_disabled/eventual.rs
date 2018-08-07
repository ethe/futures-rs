use futures::channel::oneshot;
use futures::future::{ok, err};
use futures::prelude::*;
use std::sync::mpsc;
use std::thread;

mod support;
use support::*;


#[test]
fn join1() {
    let (tx, rx) = mpsc::channel();
    ok::<i32, i32>(1).join(ok(2))
                           .map(move |v| tx.send(v).unwrap())
                           .forget();
    assert_eq!(rx.recv(), Ok((1, 2)));
    assert!(rx.recv().is_err());
}

#[test]
fn join2() {
    let (c1, p1) = oneshot::channel::<i32>();
    let (c2, p2) = oneshot::channel::<i32>();
    let (tx, rx) = mpsc::channel();
    p1.join(p2).map(move |v| tx.send(v).unwrap()).forget();
    assert!(rx.try_recv().is_err());
    c1.send(1).unwrap();
    assert!(rx.try_recv().is_err());
    c2.send(2).unwrap();
    assert_eq!(rx.recv(), Ok((1, 2)));
    assert!(rx.recv().is_err());
}

#[test]
fn join3() {
    let (c1, p1) = oneshot::channel::<i32>();
    let (c2, p2) = oneshot::channel::<i32>();
    let (tx, rx) = mpsc::channel();
    p1.join(p2).map_err(move |_v| tx.send(1).unwrap()).forget();
    assert!(rx.try_recv().is_err());
    drop(c1);
    assert_eq!(rx.recv(), Ok(1));
    assert!(rx.recv().is_err());
    drop(c2);
}

#[test]
fn join4() {
    let (c1, p1) = oneshot::channel::<i32>();
    let (c2, p2) = oneshot::channel::<i32>();
    let (tx, rx) = mpsc::channel();
    p1.join(p2).map_err(move |v| tx.send(v).unwrap()).forget();
    assert!(rx.try_recv().is_err());
    drop(c1);
    assert!(rx.recv().is_ok());
    drop(c2);
    assert!(rx.recv().is_err());
}

#[test]
fn join5() {
    let (c1, p1) = oneshot::channel::<i32>();
    let (c2, p2) = oneshot::channel::<i32>();
    let (c3, p3) = oneshot::channel::<i32>();
    let (tx, rx) = mpsc::channel();
    p1.join(p2).join(p3).map(move |v| tx.send(v).unwrap()).forget();
    assert!(rx.try_recv().is_err());
    c1.send(1).unwrap();
    assert!(rx.try_recv().is_err());
    c2.send(2).unwrap();
    assert!(rx.try_recv().is_err());
    c3.send(3).unwrap();
    assert_eq!(rx.recv(), Ok(((1, 2), 3)));
    assert!(rx.recv().is_err());
}

#[test]
fn select1() {
    let (c1, p1) = oneshot::channel::<i32>();
    let (c2, p2) = oneshot::channel::<i32>();
    let (tx, rx) = mpsc::channel();
    p1.select(p2).map(move |v| tx.send(v).unwrap()).forget();
    assert!(rx.try_recv().is_err());
    c1.send(1).unwrap();
    let (v, p2) = rx.recv().unwrap().into_inner();
    assert_eq!(v, 1);
    assert!(rx.recv().is_err());

    let (tx, rx) = mpsc::channel();
    p2.map(move |v| tx.send(v).unwrap()).forget();
    c2.send(2).unwrap();
    assert_eq!(rx.recv(), Ok(2));
    assert!(rx.recv().is_err());
}

#[test]
fn select2() {
    let (c1, p1) = oneshot::channel::<i32>();
    let (c2, p2) = oneshot::channel::<i32>();
    let (tx, rx) = mpsc::channel();
    p1.select(p2).map_err(move |v| tx.send((1, v.into_inner().1)).unwrap()).forget();
    assert!(rx.try_recv().is_err());
    drop(c1);
    let (v, p2) = rx.recv().unwrap();
    assert_eq!(v, 1);
    assert!(rx.recv().is_err());

    let (tx, rx) = mpsc::channel();
    p2.map(move |v| tx.send(v).unwrap()).forget();
    c2.send(2).unwrap();
    assert_eq!(rx.recv(), Ok(2));
    assert!(rx.recv().is_err());
}

#[test]
fn select3() {
    let (c1, p1) = oneshot::channel::<i32>();
    let (c2, p2) = oneshot::channel::<i32>();
    let (tx, rx) = mpsc::channel();
    p1.select(p2).map_err(move |v| tx.send((1, v.into_inner().1)).unwrap()).forget();
    assert!(rx.try_recv().is_err());
    drop(c1);
    let (v, p2) = rx.recv().unwrap();
    assert_eq!(v, 1);
    assert!(rx.recv().is_err());

    let (tx, rx) = mpsc::channel();
    p2.map_err(move |_v| tx.send(2).unwrap()).forget();
    drop(c2);
    assert_eq!(rx.recv(), Ok(2));
    assert!(rx.recv().is_err());
}

#[test]
fn select4() {
    let (tx, rx) = mpsc::channel::<oneshot::Sender<i32>>();

    let t = thread::spawn(move || {
        for c in rx {
            c.send(1).unwrap();
        }
    });

    let (tx2, rx2) = mpsc::channel();
    for _ in 0..10000 {
        let (c1, p1) = oneshot::channel::<i32>();
        let (c2, p2) = oneshot::channel::<i32>();

        let tx3 = tx2.clone();
        p1.select(p2).map(move |_| tx3.send(()).unwrap()).forget();
        tx.send(c1).unwrap();
        rx2.recv().unwrap();
        drop(c2);
    }
    drop(tx);

    t.join().unwrap();
}
