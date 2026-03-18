import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-marketplace-barter',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, RouterOutlet],
  templateUrl: './marketplace-barter.component.html',
  styleUrl: './marketplace-barter.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceBarterComponent {}
