# Game Frame Player  

Implemented the **GameFramePlayer** tool, which provides the following key functionalities  
to enhance the workflow in QA testing and developer troubleshooting/bug fixing:  

- **Game Frame Recording & Upload**:  
  The online game server records game frames and stores them, along with potential error logs per frame,  
  in an in-memory database. These are uploaded to cloud storage on a daily basis.  

- **Game Frame Visualization & Playback & Fast-forward**:  
  When a bug occurs in the online game, developers can use the frame player to quickly locate and visually  
  inspect the rendered game frame(s) where the error occurred.  
  This can be a specific past frame or a segment of frames from a certain time period,  
  enabling faster bug identification.  
  In addition, Players can re-watch past matches directly in the game client with playback and fast-forward features

  when user playback in frame-player client, the frame time points will be streaming to frame-player server who will:
  - interpolate the frame time points to frame index based on sliding speed. the smaller speed, the less frames will be skipped.
  - query the frame data from the in-memory cache of frame texture, otherwise query from embedded database and update the cache.
  - steaming back frame textures to the frame-player client who will render the frame textures.

 when user resume the game from the past frame, they will be asked to overwrite the current game frame or start a new play.  
 the starting frame time point will be streaming to frame-player server who will then forward frame state to game engine who will
 reset game world state to that in frame and then start game tick from there.
  
- **Manual Frame-by-Frame Tick**:  
  After identifying a bug, developers can step backward/forward the game logic frame by frame,  
  debugging anf fixing both client and server-side code.  
  They can also replay the problematic frame segments as they want to verify if the bug fix is correct.  

- **Game Frame Export & sharing**:  
  Once a bug is fixed, developers can export & share the game frames along with the patched client and server builds  
  for further QA testing.  

- **Game Frame Trim, Merge & Copy**:  
  When QA receives game frames along with the patched client and server builds,  
  they can either create new frames or copy a segment of frames from a specific time period for each new test case.

  Different test cases may require different frame segments for rendering results.  
  In such cases, QA can merge multiple frame segments for reuse.  

  If QA detects an issue in a specific frame segment, they can trim irrelevant frames,  
  keeping only the critical ones to re-produce the issue and thus reduce developer workload.  

- **Game Video Exporting**:  
  Once testing is complete, QA can export all validated game frame segments as MP4 videos for easy sharing and archiving. These videos can include overlaid text details such as the tester’s name, date, test description, and other relevant information. This feature is especially valuable for generating game-play demos during sprint reviews—helping non-technical stakeholders better visualize changes—as well as for incorporating into official test documentation. In addition, Game players also can export and download the past matches as videos.
