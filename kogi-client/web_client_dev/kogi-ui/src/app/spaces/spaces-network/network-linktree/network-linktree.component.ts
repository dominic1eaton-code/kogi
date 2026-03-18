import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-spaces-network-linktree',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './network-linktree.component.html',
  styleUrl: './network-linktree.component.css',
  host: {
    class: 'block w-full'
  }
})
export class SpacesNetworkLinktreeComponent {}
