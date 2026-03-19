import { ComponentFixture, TestBed } from '@angular/core/testing';

import { ItembookMetricsComponent } from './itembook-metrics.component';

describe('ItembookMetricsComponent', () => {
  let component: ItembookMetricsComponent;
  let fixture: ComponentFixture<ItembookMetricsComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      imports: [ItembookMetricsComponent]
    }).compileComponents();

    fixture = TestBed.createComponent(ItembookMetricsComponent);
    component = fixture.componentInstance;
    await fixture.whenStable();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
