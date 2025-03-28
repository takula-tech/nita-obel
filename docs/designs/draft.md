# `ecs introduction`

- **`Entity`**:
  - ArcheType: Container of Components
  - EntityId: A unique identifier(u32) that can be used to access components
  - Usually implemented as SparseSet [!!!show up the sparse set impls in source codes]
  
- **`Component`**:
  - Stored in continuous memory that is cache-line friendly
  - In-memory Database:
    - Entity Id ~ Primary Key
    - Entity Id ~ Foreigner Key (rust borrow rule cyclic reference is difficult to use)
    - Component ~ Partition Key
    - Component ~ Table Column
    - Component ~ Field Index

    ```sql
      SELECT hp,mp,amor from entitiesTable
      WHERE alignment = 'team_mate' AND (hp <100 OR mp >= 100)
      ORDER BY hp DESC
      GROUP BY alignment
      LIMIT 10
    ```

    ```rust
    fn find_heal_targets(entities:
      Select<
        (
          &HealthCmpt,
          &ManaCmpt,
          &ArmorCmpt,
        ),
        Where<(
          And<(
            With<AlignmentIsTeamMate>,
            Or<(
              With<HpGreaterThan20TagCmpt>,
              With<MpGreaterThanOrEqualTo100TagCmpt>,
            )>
          )>
        )>,
        OrderBy<(HealthCmpt, Desc)>,
        GroupBy<AlignmentCmpt>,
        Limit<10>,
      >
    ) 
    {
      for (health, mana, armor) in entities.iter() {
          println!("Health: {}, Mana: {}, Armor: {}", health, mana, armor);
      }
    }
    ```

- **`System`**:
  - Naturally follow the SRP & OCP
  - Naturally have better cpu branch prediction
      avoid the use of likely()/unlikely() in c++

      ```rust
        fn update_entities(entities: &mut [Entity]) {
            for entity in entities.iter_mut() {
                if entity.is_mob() {
                    ...
                } else if entity.is_player() {
                    ...
                }
                else if entity.is_stunned() {
                    ...
                } 
                ...
            }
        }

        fn update_mob(mobs:
          Select<(&mut mobs,),Where<(With<IsMobTagCmpt>,)>>
        ) {
            for mob in mobs.iter_mut() {
              ...
            }
        }

        fn update_players(players:
          Select<(&mut players,),Where<(With<IsPlayerTagCmpt>,)>>
        ) {
            for player in players.iter_mut() {
              ...
            }
        }
      ```
  
  - Invoke System
    - via ThreadPool Executor (pre-defined)
      - Pre-register Systems to execute before app runs
      - Acyclic Graph of Systems (Parallels & Sequential)

    - via Event (On the Fly)
      - UI button clicked 3 times

      - Call/Notify other interested systems from CatchUIInteraction system ?
      - Traditional OOP:
          CatchUIEvent object holds the references to the interested objects  
          and invoke onClicked method on those objects
      - We cannot simply call other interested systems  
          from CatchUIEvent system that causes  
          data racing, workload imbalance,  
          stack overflow due to cyclic invoke loop

      - what can help here =>  
          `Invoke System via event on the fly`
      - Helps workload peak

      - what if we need to get something returned from the system ?
      - await helps here ? -> No unfortunately
      - await will break the ordering constraint between systems
             and is not suitable for this case as we already been using ThreadPool Executor

      - what if we need make animation that need yield&await system ?

      - what if event handling logics vary when:
        - they're different entity archetypes
        - even though same archetype, but still depend on on-the-fly conditions/context
        - in other words, instead of simply stateless sequential logics,
            we have complex stateful orchestration logics
            eg, in early local dev loop, we do not know wjat exactly

          ```rust
          struct UIClickedEvent<Payload> {
            eventType: u32,
            from: EntityID,
            to: EntityID[32],
            payload: Payload,
          }

          fn system_to_handle_click_event(entityStore: EntityStore, events: EventBus<(UIClickedEvent,)>) {
            for event in events.iter()
            {
              if let Some(fromEntity) = entityStore.get(event.from)
              {
                if fromEntity.hasCmpt<IsFormSubmissionButton>.is_some() {
                  validate_form_errors();
                  let subResult = submit_form().await;
                } else if fromEntity.hasCmpt<IsFormCloseButton>.is_some() {
                    ...
                } else if fromEntity.hasCmpt<IsFormCloseButton>.is_some() {
                    ...
                }
                ...
              }
            }
          }
          ```

      - What can help here => `Actor-based State Chart`

        - Higher-level construct implemented by ECS
            to model hierarchical state machines (`HSM`)
          - Event driven state transitions (State Machine Cmpt)
          - Event driven intra-chart communications (Mailbox Cmpt)

        - Designated for complex Sync/Async logics
            eg, `yield` and `awaiting` features that are must-have high level  
            construct in dev loop for 2d UI, animation & Game Plays  
            which are essentially state machines.
98
          - animation of cascading dropdown menu that requires controls the timeline  
               for each individual dropdown item
          - game skill

            yield=> historical statechart
            awaiting

        - Designated for complex interactions across entities
          - publish subscription
          - direct messaging
        - Good readability & testability and usability

## `2D UI Render Tree`

- **`2D Render Node/Entity`**:

     ```rust

      #[Component]
      enum EntityLinkCmpt {
        entityID: EntityID,
        parents: vec[EntityID],
        children: vec[EntityID],
      }

      #[Component]
      struct 2DUILayoutStyleCmpt {
        LayoutStyle: TextLayoutStyle,
      }

      #[Component]
      struct TextLayoutStyleCmpt {
        textLayoutStyle: TextLayoutStyle,
      }

      #[Component]
      struct TextCharsCmpt {
        textChars: vec<char>,
      }

      #[ComponentBundle]
      struct 2DUITextBundle {
        textLayoutStyleCmpt: TextLayoutStyleCmpt,
        textCharsCmpt: TextCharsCmpt,
      }

      #[Component]
      enum VisibilityCmpt {
        visible,
        invisible
      }

      #[RenderNodeBundle]
      struct RenderNodeBundle {
        vertexBuffer: VertexBuffer,
        texture: TextCharsCmpt,
        entityLinkCmpt: EntityLinkCmpt
      }

      #[ComponentBundle]
      struct UIButtonBundle {
        uiLayoutStyleCmpt: 2DUILayoutStyleCmpt,
        textBundle: 2DUITextBundle,
        stateChartCmpt: StateChartCmpt,
        entityLinkCmpt: EntityLinkCmpt,
      }
     ```

- **`Tree Construction`**:
  - Similar toReact Component design patterns

    ```rust


    ```

- **`Dynamic Tree Manipulation`**:
  - Tree Creation

     ```rust
      create_entity()
     ```

  - Deletion: delete the render node and all its descendants nodes
  - Update:

- **`Data Flow`**:
  - Event driven render tree create/update app logics in state chart cmpt to react to UI/Application events
  
- **`UI Layout Recalculation`**
  - [taffy-low-level-api](https://docs.rs/taffy/latest/taffy/#low-level-api)

- **`Text Layout`**
  
## `2D UI Render Pipeline`
