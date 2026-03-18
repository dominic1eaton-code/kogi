import { Component } from '@angular/core';
import { RouterLink } from '@angular/router';

@Component({
  selector: 'app-marketplace-exchange-commodities',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './marketplace-exchange-commodities.component.html',
  styleUrl: './marketplace-exchange-commodities.component.css',
  host: {
    class: 'block w-full'
  }
})
export class MarketplaceExchangeCommoditiesComponent {}
