One-time setup (if not done yet):

  cd /home/jan/cc_side_projects/fullstack_project_template                                                                                                                                   
  ./scripts/setup.sh        # installs npm deps, runs pre-commit install, copies .env                                                                                                        
                                                                                                                                                                                             
  Then edit .env — the JWT_SECRET placeholder is intentionally long but you should replace it with a real random value:                                                                      
                                                                                                                                                                                             
  # generate a proper secret                                                                                                                                                                 
  openssl rand -base64 32                                                                                                                                                                  
  # paste it into .env as JWT_SECRET=... 
                                                                                                                                                                                             
  Start dev servers:
                                                                                                                                                                                             
  ./scripts/dev.sh                                                                                                                                                                         
                                                                                                                                                                                             
  This runs the Rust backend on http://localhost:3000 and the Vite frontend on http://localhost:5173. The frontend proxies /api/v1/ to the backend automatically.                            
                                                                                                                                                                                             
  Manual flow to verify it works:                                                                                                                                                            
                                                                                                                                                                                           
  1. Open http://localhost:5173          
  2. Register an account at /register
  3. Log in at /login → should redirect to /dashboard                                                                                                                                        
  4. Create a data item on the dashboard                                                                                                                                                     
  5. For password reset: submit your email at /forgot-password, then watch the backend terminal — it prints the reset URL to the console (no real email needed in dev)                       
                                                                                                                                                                                             
  Run tests:                                                                                                                                                                               
                                                                                                                                                                                             
  ./scripts/test.sh         # cargo test + npm test                                                                                                                                        
  ./scripts/lint.sh         # clippy, rustfmt, eslint, prettier, svelte-check                                                                                                                
                                                                                                                                                                                             
  Or with Docker (closest to production):                                                                                                                                                    
                                                                                                                                                                                             
  docker compose up --build                                                                                                                                                                  
  # app available at http://localhost:3000                                                                                                                                                 
                                                                                                                                                                                             
  Note: Docker requires JWT_SECRET to be set in your .env file since we removed the default from docker-compose.yml.
