import { Component, inject } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet, Router } from '@angular/router';
import { NavigationComponent } from '../index/navigation/navigation.component';

@Component({
  selector: 'app-hub',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, RouterOutlet, NavigationComponent],
  templateUrl: './hub.component.html',
  styleUrl: './hub.component.css',
  host: {
    class: 'block w-full min-h-screen'
  }
})
export class HubComponent {
  // Inject the Router service
  private router = inject(Router);
  
  // Method to handle the navigation
  logout(): void {
    // Perform any necessary logic here
    console.log('Logging out of client session...');

    // Navigate to the specified route
    this.router.navigate(['/login']);
  }}
