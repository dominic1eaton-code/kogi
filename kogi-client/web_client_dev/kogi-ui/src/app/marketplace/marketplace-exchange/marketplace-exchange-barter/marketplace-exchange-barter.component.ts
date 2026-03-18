import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-marketplace-exchange-barter',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './marketplace-exchange-barter.component.html',
  styleUrl: './marketplace-exchange-barter.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceExchangeBarterComponent {}
