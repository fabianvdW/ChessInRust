pub const DEFAULT_MOVE_OVERHEAD: u64 = 25;
pub const MIN_MOVE_OVERHEAD: u64 = 0;
pub const MAX_MOVE_OVERHEAD: u64 = 20000;

pub struct TimeControlInformation {
    pub stable_pv: bool,
    pub high_score_diff: bool,
}

impl TimeControlInformation {
    pub fn new(time_saved: u64) -> Self {
        TimeControlInformation {
            stable_pv: false,
            high_score_diff: false,
        }
    }
}

#[derive(Clone, Copy)]
pub enum TimeControl {
    Incremental(u64, u64),
    MoveTime(u64),
    Infinite,
    Tournament(u64, u64, usize),
}

impl TimeControl {
    pub fn to_go(&self, white: bool) -> String {
        match &self {
            TimeControl::Incremental(time_left, inc) => {
                if white {
                    format!("wtime {} winc {}", time_left, inc)
                } else {
                    format!("btime {} binc {}", time_left, inc)
                }
            }
            TimeControl::MoveTime(time) => format!("movetime {}", time),
            TimeControl::Infinite => "infinite".to_owned(),
            TimeControl::Tournament(time_left, inc, movestogo) => {
                if white {
                    format!("wtime {} winc {} movestogo {}", time_left, inc, movestogo)
                } else {
                    format!("btime {} binc {} movestogo {}", time_left, inc, movestogo)
                }
            }
        }
    }
    pub fn update(&mut self, time_spent: u64, tournament_info: Option<(usize, u64)>) {
        match self {
            TimeControl::Incremental(left, inc) => {
                assert!(*left > time_spent);
                *self = TimeControl::Incremental(*left - time_spent + *inc, *inc);
            }
            TimeControl::MoveTime(time) => {
                *self = TimeControl::MoveTime(*time);
            }
            TimeControl::Infinite => panic!("Should not call updat eon Infinite"),
            TimeControl::Tournament(left, inc, movestogo) => {
                assert!(*left > time_spent);
                let mut new_left = *left - time_spent + *inc;
                if *movestogo == 0 {
                    new_left += tournament_info.unwrap().1;
                    *movestogo = tournament_info.unwrap().0;
                }
                *self = TimeControl::Tournament(new_left, *inc, *movestogo);
            }
        }
    }
    pub fn time_left(&self) -> u64 {
        match self {
            TimeControl::Incremental(left, _) => *left,
            TimeControl::MoveTime(left) => *left,
            TimeControl::Infinite => panic!("Should not call time_left on Infinite"),
            TimeControl::Tournament(left, _, _) => *left,
        }
    }
    fn get_normal_tc_info(&self) -> (u64, u64, usize) {
        match self {
            TimeControl::Incremental(mytime, myinc) => (*mytime, *myinc, 30),
            TimeControl::Tournament(mytime, myinc, movestogo) => (*mytime, *myinc, *movestogo),
            _ => panic!("Only call this function on normal timecontrols"),
        }
    }
    pub fn time_over(&self, time_spent: u64, tc_information: &TimeControlInformation, move_overhead: u64) -> bool {
        if let TimeControl::Infinite = self {
            return false;
        } else if let TimeControl::MoveTime(move_time) = self {
            return time_spent > move_time - move_overhead || *move_time < move_overhead;
        } else {
            let (mytime, myinc, movestogo) = self.get_normal_tc_info();
            if time_spent as isize > mytime as isize - 4 * move_overhead as isize {
                return true;
            }
            let normal_time = ((mytime - tc_information.time_saved) as f64 / movestogo as f64) as u64 + myinc - move_overhead;
            let time_aspired = if tc_information.time_saved < normal_time {
                ((normal_time as f64 * 0.85) as u64).max(myinc)
            } else {
                normal_time.max(myinc)
            };
            if time_spent < time_aspired {
                return false;
            }
            if tc_information.stable_pv {
                return true;
            }
            //Non stable pv so we increase time
            return time_spent as f64 > 1.15 * (normal_time + tc_information.time_saved) as f64;
        }
        panic!("Invalid Timecontrol");
    }

    pub fn time_saved(&self, time_spent: u64, saved: u64, move_overhead: u64) -> i64 {
        match self {
            TimeControl::Incremental(_, _) | TimeControl::Tournament(_, _, _) => {
                let (mytime, myinc, movestogo) = self.get_normal_tc_info();
                let normal_tc = ((mytime - saved) as f64 / movestogo) as u64 + myinc - move_overhead;
                normal_tc as i64 - time_spent as i64
            }
            _ => 0,
        }
    }

    pub fn as_string(&self, tc_information: &TimeControlInformation, move_overhead: u64) -> String {
        let mut res_str: String = String::new();
        if let TimeControl::MoveTime(time) = self {
            res_str.push_str(&format!("Limited movetime: {}\n", time));
        } else if let TimeControl::Infinite = self {
            res_str.push_str("Infinite Time!\n");
        } else {
            let (mytime, myinc, movestogo) = self.get_normal_tc_info();
            res_str.push_str(&format!("My Time: {}\n", mytime));
            res_str.push_str(&format!("My Inc: {}\n", myinc));
            res_str.push_str(&format!("Moves to go : {}\n", movestogo));
            let normal_time = ((mytime as f64 - tc_information.time_saved as f64) / movestogo as f64) as u64 + myinc - move_overhead;
            let time_aspired = if tc_information.time_saved < normal_time {
                ((normal_time as f64 * 0.85) as u64).max(myinc)
            } else {
                normal_time.max(myinc)
            };
            res_str.push_str(&format!("My normal time I would spend: {}\n", normal_time));
            res_str.push_str(&format!("My aspired time I would spend: {}\n", time_aspired));
        }
        res_str
    }
}
