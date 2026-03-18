import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-marketplace-market',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, RouterOutlet],
  templateUrl: './marketplace-market.component.html',
  styleUrl: './marketplace-market.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceMarketComponent {}
