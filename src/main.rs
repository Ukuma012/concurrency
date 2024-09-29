use std::ptr::{read_volatile, write_volatile};
use std::sync::atomic::{fence, Ordering};
use std::thread;

const NUM_THREADS: usize = 4;
const NUM_LOOP: usize = 100000;

static mut LOCK: BakeryLock = BakeryLock {
    entering: [false; NUM_THREADS],
    tickets: [None; NUM_THREADS]
};

// volatile用のマクロ
macro_rules! read_mem {
    ($addr: expr) => {
        unsafe {
            read_volatile($addr)
        }
    };
}
macro_rules! write_mem {
    ($addr: expr, $val: expr) => {
       unsafe {
        write_volatile($addr, $val)
       } 
    };
}

struct BakeryLock {
    // その要素番号のスレッドが現在チケットを取得中かどうかを示す配列
    entering: [bool; NUM_THREADS],
    // その要素番号のスレッドが持つチケットに書かれた番号を示す配列
    // i番目のスレッドのチケットはtickets[i]で取得できる
    tickets: [Option<u64>; NUM_THREADS],
}

struct LockGuard {
    idx: usize
}

impl Drop for LockGuard {
    fn drop(&mut self) {
        fence(Ordering::SeqCst);
        write_mem!(&mut LOCK.tickets[self.idx], None);
    }
}

impl BakeryLock {
    //idxはスレッド番号
    fn lock(&mut self, idx: usize) -> LockGuard {
        //チケット獲得処理
        fence(Ordering::SeqCst);
        write_mem!(&mut self.entering[idx], true);
        fence(Ordering::SeqCst);

        // 現在配布されているチケットの最大値を取得
        let mut max = 0;
        for i in 0..NUM_THREADS {
            if let Some(t) = read_mem!(&self.tickets[i]) {
                max = max.max(t);
            }
        }

        let ticket = max + 1;
        write_mem!(&mut self.tickets[idx], Some(ticket));

        fence(Ordering::SeqCst);
        write_mem!(&mut self.entering[idx], false);
        fence(Ordering::SeqCst);

        for i in 0..NUM_THREADS {
            if i == idx {
                continue;
            }

            while read_mem!(&self.entering[i]){}

            loop {
                // スレッドiと自分の優先順位を比較して
                // 自分の方が優先順位が高いか、
                // スレッドiが処理中でない場合に待機を終了
                match read_mem!(&self.tickets[i]) {
                    Some(t) => {
                        if ticket < t || (ticket == t && idx < i) {
                            break;
                        }
                    }
                    None => {
                        break;
                    }
                }
            }
        }

        fence(Ordering::SeqCst);
        LockGuard{idx}

    }
}

static mut COUNT: u64 = 0;

fn main() {
    let mut v = Vec::new();
    for i in 0..NUM_THREADS {
        let th = thread::spawn(move || {
            for _ in 0..NUM_LOOP {
                let _lock = unsafe {LOCK.lock(i)};
                unsafe {
                    let c = read_volatile(&COUNT);
                    write_volatile(&mut COUNT, c+1);
                }
            }
        });
        v.push(th);
    }

    for th in v {
        th.join().unwrap();
    }

    println!(
        "COUNT = {} (expexted = {})",
        unsafe {COUNT},
        NUM_LOOP * NUM_THREADS
    );
}