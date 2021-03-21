# Runger = "Rust" + "Hunger"

Runger is a hunger games style simulation game (?) written in Rust (hence the name). Despite the simple 2D graphics and a game-like nature, the initial versions of Runger are supposed to have little to no ability for the user/observer to influence what happens in the simulation.

## The point and the concept

The main point of this project is to experiment with / research the concept of [genetic programming](https://en.wikipedia.org/wiki/Genetic_programming) and simulation of generations of in-game "players". The players usually start with close to nothing, and their main objective is to survive: they have to kill other "players" and compete for resources (such as food). The drive to act may be hunger. The point of the simulation is to find the player with the perfect set of genes. Players operate in generations (one generation lasts until everyone is dead). After a generation, the most "successful" players get chosen for production of the next generation, which may involve crossing them with each other ("breeding") and an injection of random genes (mutations). After that, the next generation starts. And ideally this should last until the "perfect specimens" are found, that is, ones that never die.

## Experimental nature of the project

This project is ridiculously experimental. It's intended to be developed and maintained for a relatively long time (at least several months, potentially more if it goes well), but it's being built with Rust + Amethyst. While Rust itself is a relatively mature language (with a very young infrastructure though), Amethyst is a very new crate/game engine and it's not in any way feature-complete or even stable yet. This project is intended to experiment with new technology just as much as with the genetic programming concept. I have no clue where this will end up in a few months from now (today, at the time of writing this, is 03/21/2021), so we'll see.

## Future plans

The project has just started. Even though it's intended to be developed/maintained for quite a while, I'm a single dev who's working on this thing, and my knowledge of Rust /genetic programming / 2D graphics / game dev is also extremely limited. This is also a secondary project of mine (working on it for approximately 2 days a week). Therefore, the progress is going to happen, but it's going to most likely be slow.

The plans for now are to release the 0.1.0 version. The plans for that are modest: for 0.1.0, I just want to have the board with players/agents, and they should be taking a single turn (making their move once) when you press space. Maybe they'll also going to have the ability to attack other players and try to find food. That is pretty much it. All the generations production / breeding / finding the fittest specimens is going to come in later versions, this is just for the very basics.

To know what's going on with Runger currently and where it is moving, please look at the [milestones](https://github.com/Oleksii-Kshenskyi/runger/milestones) page. The most recent milestone is usually the one being worked on.
- 0.1.0 is for the basic graphics/board development/players and resources development.
- 0.2.0 is for the genetics part of the application.
- 0.3.0 is maybe going to introduce some ability for the human player/observer/user to influence the simulation in some way. Also, refining the genetic algorithm and introducing new systems (maybe new resources, or evolving new abilities for the players) is planned for around this version.
- 0.4.0 and beyond are going to focus on refining the existing systems in the simulation and potentially introducing new interesting features.

## Building and running

As usual with Rust projects, to build it run `cargo build`, and to run it `cargo run`.
When/if any new build conditions are introduced, they will be reflected here in this README section.

## Licensing

Runger is licensed under [The Unlicense](https://unlicense.org/). This means that Runger is public domain and anyone can do whatever the heck they want with it. And no, you don't owe me to redistribute any copyright notices or anything. Public domain means this software effectively doesn't belong to me, so no copyrights are possible. It's yours to do whatever you want with it, without any obligations (not even the minimal ones), and that's it.