This sets the item to be returned.

This function can be used for both OK and Err, and actually stores the item.

However, if the item has already been added (i.e., if it was already created using [`Iart::with_item`] or [`Iart::new_ok`]), it will be overwritten.