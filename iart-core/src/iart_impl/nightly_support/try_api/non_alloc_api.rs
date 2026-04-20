use crate::IartErr;
use crate::events::{AutoRequestType, IartEvent};
use crate::types::Iart;
use core::convert::Infallible;
use core::fmt::Debug;
use core::ops::{ControlFlow, FromResidual, Try};

impl<Item> Try for Iart<Item>
where
    Item: Debug,
{
    type Output = Item;
    type Residual = Iart<Infallible>;

    #[inline]
    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }

    #[track_caller]
    fn branch(mut self) -> ControlFlow<Self::Residual, Self::Output> {
        self.send_log();

        let _ = unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::TryUsed))
                .unwrap_unchecked()
        };

        match self.data.take() {
            Some(Ok(item)) => {
                self.handled = true;
                ControlFlow::Continue(item)
            }
            Some(Err(err)) => {
                let res: Iart<Infallible> = self;
                res
            }
            None => panic!("Iart: try branch called after consumption"),
        }
    }
}

impl<Item> FromResidual<Iart<Infallible>> for Iart<Item>
where
    Item: Debug,
{
    #[track_caller]
    fn from_residual(mut residual: Iart<Infallible>) -> Self {
        residual.send_log();
        residual.handled = true;

        let data = residual.data.take();
        #[cfg(feature = "allow-backtrace-logging")]
        let log = residual.log.take();

        Self {
            data: data.map(|r| Err(unsafe { r.unwrap_err_unchecked() })),
            handled: false,
            #[cfg(feature = "allow-backtrace-logging")]
            log,
            #[cfg(feature = "error-can-have-item")]
            err_item: residual.err_item.take(),
            trans_fns: residual.trans_fns,
        }
    }
}

impl<Item, E> FromResidual<Result<Infallible, E>> for Iart<Item>
where
    E: IartErr + 'static,
{
    #[track_caller]
    fn from_residual(residual: Result<Infallible, E>) -> Self {
        let err = unsafe { residual.unwrap_err_unchecked() };

        Self::Err(err, None)
    }
}

impl<Item> core::ops::Residual<Item> for Iart<Infallible>
where
    Item: Debug,
{
    type TryType = Iart<Item>;
}
