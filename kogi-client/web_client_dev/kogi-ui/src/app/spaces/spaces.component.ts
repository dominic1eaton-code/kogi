import { Component, inject } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet, Router } from '@angular/router';

@Component({
  selector: 'app-spaces',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, RouterOutlet],
  templateUrl: './spaces.component.html',
  styleUrl: './spaces.component.css',
  host: {
    class: 'block w-full min-h-screen'
  }
})
export class SpacesComponent {

  // Inject the Router service
  private router = inject(Router);
  
  // Method to handle the navigation
  logout(): void {
    // Perform any necessary logic here
    console.log('Logging out of client session...');

    // Navigate to the specified route
    this.router.navigate(['/login']);
  }
}
