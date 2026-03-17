import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

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
export class SpacesComponent {}
