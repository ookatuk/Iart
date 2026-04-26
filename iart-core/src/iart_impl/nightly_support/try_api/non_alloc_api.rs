#![doc = include_str!("../../../../doc/modules/non_alloc_api.md")]

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
    #[doc = include_str!("../../../../doc/fn/Iart/branch.md")]
    fn branch(mut self) -> ControlFlow<Self::Residual, Self::Output> {
        self.send_log();

        let _ = unsafe {
            self.send_log_to_handler::<true>(IartEvent::FunctionHook(AutoRequestType::TryUsed))
                .unwrap_unchecked()
        };

        if self.is_ok().unwrap_or(false) {
            self.handled = true;
            ControlFlow::Continue(self.data.take().unwrap().unwrap())
        } else {
            let clos = |_| -> Infallible { unreachable!() };

            #[cfg(feature = "error-can-have-item")]
            let res: Iart<Infallible> = self.map_err_item(clos, clos);

            #[cfg(not(feature = "error-can-have-item"))]
            let res: Iart<Infallible> = self.map(clos);

            ControlFlow::Break(res)
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

        residual.map(|_| unreachable!())
    }
}

#[cfg(not(feature = "no-alloc"))]
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

#[cfg(feature = "no-alloc")]
impl<Item, E> FromResidual<Result<Infallible, &'static E>> for Iart<Item>
where
    E: IartErr + 'static,
{
    #[track_caller]
    fn from_residual(residual: Result<Infallible, &'static E>) -> Self {
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
