import { Component } from '@angular/core';
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router';

@Component({
  selector: 'app-marketplace-exchange',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, RouterOutlet],
  templateUrl: './marketplace-exchange.component.html',
  styleUrl: './marketplace-exchange.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceExchangeComponent {}
