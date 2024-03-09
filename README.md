# Runger = "Rust" + "Hunger"

Runger is a hunger games style simulation game (?) written in Rust (hence the name). Despite the simple 2D graphics and a game-like nature, the initial versions of Runger are supposed to have little to no ability for the user/observer to influence what happens in the simulation.

## The point and the concept

The main point of this project is to experiment with / research the concept of [genetic programming](https://en.wikipedia.org/wiki/Genetic_programming) and simulation of generations of in-game "players". The players usually start with close to nothing, and their main objective is to survive: they have to kill other players and compete for resources (such as food). The drive to act may be hunger. The point of the simulation is to find the player with the perfect set of genes. Players operate in generations (one generation lasts for a certain number of turns). After a generation, the most "successful" players get chosen for production of the next generation, which may involve crossing them with each other ("breeding") and an injection of random genes (mutations). After that, the next generation starts. And ideally this should last until the "perfect specimens" are found, that is, ones that never die.

## Experimental nature of the project

This project is ridiculously experimental. It's intended to be developed and maintained for a relatively long time (at least several months, potentially more if it goes well), but it's being built with Rust + Bevy. While Rust itself is a relatively mature language (with a very young infrastructure though), Bevy is a new crate/game engine and is changing rapidly with each version. This project is intended to experiment with new technology just as much as with the genetic programming concept. 

Once the "hunger games" simulation is finished, I may start adding other types of simulation just to see if the players can adjust to radically different conditions of their worlds. Therefore, this project potentially doesn't have an end date. Once I figure out where to move next, I'll update this README. For now, I'm focused on the Hunger Games implementation.

## Future plans

The project has just started. Even though it's intended to be developed/maintained for quite a while, I'm a single dev and due to the above-mentioned experimental nature of the project, just about everything is an experiment with this project.

The plans for now are to release the 0.1.0 version. The plans for that are modest: for 0.1.0, we'll have some number of players on the board, they kill, roam around looking for food, and try to survive. At the end of the generation, they're crossed with the other surviving players, the new gen is produced and those guys start playing instead. We'll also be able to simulate a number of generations and only enable visualization/graphics to show the nth generation (once they've evolved and got a bit smarter than the first random ones were), therefore showcasing the genetic evolution. That's it for the first release. 

To know what's going on with Runger currently and where it is moving, please look at the [milestones](https://github.com/Oleksii-Kshenskyi/runger/milestones) page. The most recent milestone is usually the one being worked on.
- 0.1.0 is for the basic graphics/board development/players and resources development, as well as the basics of genetics/evolution and non-graphical simulation of several generations.
- Future versions are going to focus on improving the existing simulation, adding features and potentially developing different kinds of simulations and win conditions, other than just hunger games and "try not to die".

## Building and running

As usual with Rust projects, to build it run `cargo build`, and to run it `cargo run`.
When/if any new build conditions are introduced, they will be reflected here in this README section.

## Licensing

Runger is licensed under [The Unlicense](https://unlicense.org/). This means that Runger is public domain and anyone can do whatever the heck they want with it. And no, you don't owe me to redistribute any copyright notices or anything. Public domain means this software effectively doesn't belong to me, so no copyrights are possible. It's yours to do whatever you want with it, without any obligations (not even the minimal ones), and that's it.

However, in the future the licensing on this may change. If the project gets off the ground and becomes a fairly complex simulation of evolution, I may end up picking an opensource-only license like GPLv2 to encourage contribution and open nature of the project.