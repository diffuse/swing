# Contributing

Thank you for making/considering a contribution to this project!  These guidelines are not rules, but are designed to help us keep this project easily maintainable and consistent.

## Commit messages

Ideal commit messages give a high level descripition of your change, and are written in [the imperative mood](https://en.wikipedia.org/wiki/Imperative_mood).  For example:

```shell
# this is in the imperative mood
$ git commit -m 'Add a cool new feature'

# this is not
$ git commit -m 'Added a cool new feature'

# this is also not
$ git commit -m 'Cool new feature'
```

Commit messages written this way make it easy to look at the `git log` and quickly grasp major actions/changes that take us from one commit to another. 

## Styling and testing

Before making a pull request, please format your changes and make sure all tests still pass:

```shell
$ cargo fmt
$ cargo test
```