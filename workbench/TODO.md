# TODO

## Active

## Backlog

- [ ] The method `get_mei_with_options` is suuuper awkward; let's change `get_mei` to support options ... 'optionally' so that the user only ever has to use the `get_mei` method.
- [ ] The `get_page_*`, `get_element_*`, `get_time_*`, etc., methods are also really awkward; we need to support a cleaner `get_<name>` methods with options that does the same thing, just more ergonomically.

## Completed

- [x] Add trait for `render` like we did for the trait that allowd us to add the `load` method (what is the full list of `render*` methods?)
- [x] Add an option for the new `render` method that will dispatch to `render_all_pages`
