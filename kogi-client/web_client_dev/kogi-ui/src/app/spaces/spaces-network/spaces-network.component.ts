import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-spaces-network',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, RouterOutlet],
  templateUrl: './spaces-network.component.html',
  styleUrl: './spaces-network.component.css',
  host: {
    class: 'block w-full'
  }
})
export class SpacesNetworkComponent {}
