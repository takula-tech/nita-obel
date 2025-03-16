## Skill BerserkerCall

Axe taunts nearby enemy units, forcing them to attack him at high speed,

while he gains bonus armor during the duration.

```bash
RADIUS: 315
BONUS ARMOR: 16 / 19 / 22 / 25
DURATION: 1.8 / 2.2 / 2.6 / 3
CAST POINT: 0.3s 17 / 15 / 13 / 11
MANA: 80 / 90 / 100 / 110
```

firstly we need validate if this skill is meet any constraints,

eg, if axe has enough mana, if he is stunned and more.

As those states located at another actor of state store,
the value we query might be stale. eg, we query the mana
is now 100 and enough for the skill cost of 30. but later on
we could fail decrement mana by 30 due to the current
mana value when state store received the msg is >= 30.

this problem will becomes more complex if we succeed
to update some value 1 but failed to the value 2 that exist
in different actors.

at that case, we need also worry about the state rollback.
this is typical data inconsistency issue in any parallel system.

the actor model only makes sure no data race but not solve
data inconsistency issue because it can only be solved correctly
at application level who knows what are the correct mutation
sequences.

we tag each msg with frame number to avoid too
many rewind&replay when msg coming with messed ordering.
so the msgs with same frame number are treated as happing
at the same time and no ordering constraint.

actor provides 2 communication pattern:

Tell: send msg to to dest actor, not await acknowledge.

Ask: send msg to dest actor, await for acknowledge

when asking something, the caller execution is blocked
until receiving response from callee actor.

Note that cpu is not blocked but switch to do other things. it is same thing
to promise in nodejs and task in csharp fro async.

To achieve it, actor supports re-entre by using await syntax candy provided by language.
so anyone you want to wait something, use await to block the current execution of msg
in actor mailbox, which creates a critical section where no other msgs will enter the section
and potentially mutate the states.

In such case, the traditional thread lock might be needed to protect the
mutation of shared data if you use something like promise.all(xxx) that will mutate shared
states.

see [here](./Axe.drawio) for detail interactions.
