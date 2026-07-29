use anyhow::{Context, Result};
use rustix::{
    fs::Timespec,
    time::{
        Itimerspec, TimerfdClockId, TimerfdFlags, TimerfdTimerFlags, timerfd_create,
        timerfd_settime,
    },
};
use std::os::fd::{AsFd, BorrowedFd, OwnedFd};

pub(crate) struct Timer {
    fd: OwnedFd,
}

impl Timer {
    pub(crate) fn new() -> Result<Self> {
        let fd = timerfd_create(TimerfdClockId::Monotonic, TimerfdFlags::NONBLOCK)
            .context("timerfd_create() failed")?;
        timerfd_settime(
            &fd,
            TimerfdTimerFlags::ABSTIME,
            &Itimerspec {
                it_interval: Timespec {
                    tv_sec: 1,
                    tv_nsec: 0,
                },
                it_value: Timespec {
                    tv_sec: 1,
                    tv_nsec: 0,
                },
            },
        )
        .context("timerfd_settime() failed")?;
        Ok(Self { fd })
    }

    pub(crate) fn read(&self) -> Result<()> {
        let mut buf = [0_u8; 8];
        let bytes_read = rustix::io::read(&self.fd, &mut buf)?;
        assert_eq!(bytes_read, 8);
        Ok(())
    }
}

impl AsFd for Timer {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.fd.as_fd()
    }
}
