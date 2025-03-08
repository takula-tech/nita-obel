# `ecs introduction`
 - memory layout: data is stored in continuous memory that is cache-line friendly
 - data query: avoid branch check that is cpu-branch-prediction friendly.
   eg, if(entities[i].isMob) updateMob(data[i]) else updatePlayer(data[i])
   avoid the use of likely()/unlikely() in c++.
   component store is like database where query on each component is fast becaue each filed has own index
   single field and composite index
   partitionkey = time or number as partition to query range of data eg all player with hp bettwen 20%-30%

   select entities where hp > 20 and hp < 30 and money > 10000 and type = 'melee'
   material view
 - SYSTEM: 
   when to use event: 1 cross domain event 2 short lived derived data,logn lived is often the entity component data
   3 avoid performanc peak - 
   thread scheduler: 



